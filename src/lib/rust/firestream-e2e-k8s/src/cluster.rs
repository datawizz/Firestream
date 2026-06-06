//! Per-test k3d cluster lifecycle.
//!
//! [`create_cluster`] stands up a fresh, minimum-footprint k3d cluster
//! named `firestream-e2e-<chart>-<6-char-rand>` (1 server, 0 agents,
//! built-in registry, no LB ports). The 6-char random suffix prevents
//! collisions between parallel `cargo test` invocations across
//! processes; the harness-level mutex (`harness::run_one`) handles the
//! in-process case.
//!
//! [`ClusterGuard`] tears the cluster down on Drop under a 60s budget.
//! Honors `FIRESTREAM_E2E_K8S_KEEP=1` (logs and returns instead of
//! deleting) so operators can inspect a failing cluster post-test.

use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use firestream_e2e_core::guard::wait_with_budget;
use k8s_manager::{
    K3dClusterConfig, K3dClusterManager, K3dDevModeConfig, K3dNetworkConfig, K3dRegistryConfig,
    K3dTimeoutConfig, K3dTlsConfig,
};
use rand::Rng;
use rand::distributions::Alphanumeric;
use tracing::warn;

/// Handle to a live per-test k3d cluster. Cheap to `Clone` so the
/// guard can own one while the test still holds another for kubectl
/// access.
#[derive(Debug, Clone)]
pub struct ClusterHandle {
    /// Cluster name as known to `k3d` (without the `k3d-` prefix that
    /// k3d uses for container/network names).
    pub name: String,
    /// Path to a kubeconfig pinned to this cluster only. Hand this to
    /// `KubectlExec` or pass via `kubectl --kubeconfig`.
    pub kubeconfig: PathBuf,
    /// Registry container name (matches what `k3d` emits for the
    /// built-in registry).
    pub registry: String,
}

/// Generate a 6-char lowercase alphanumeric suffix. Cluster names
/// must match `[a-z0-9-]+`, and timestamps are too predictable (and
/// too long) for a per-test name — random is better.
fn random_suffix() -> String {
    let mut rng = rand::thread_rng();
    // `Alphanumeric` over `&[u8]` gives `[A-Za-z0-9]`; lowercase the
    // result so the resulting cluster name is k8s-DNS-safe.
    (0..6)
        .map(|_| {
            let b: u8 = rng.sample(Alphanumeric);
            (b as char).to_ascii_lowercase()
        })
        .collect()
}

/// Reserve an ephemeral TCP port by binding `127.0.0.1:0`, reading the
/// kernel-assigned port, and releasing the listener. There's a small
/// race window between release and the consumer (k3d / k8s-manager's
/// registry health-check) binding, but for the lifetime of a single
/// per-test cluster it's good enough — collisions would manifest as a
/// cluster-create failure, which the test harness reports clearly.
///
/// We need this because `K3dClusterConfig`'s api_port / http_port /
/// https_port / registry.port are `u16` (not `Option<u16>`) and
/// `setup_registry` does `localhost:<port>/v2/` health-checks — so
/// `0` cannot be passed through as "let k3d auto-pick".
fn pick_ephemeral_port() -> Result<u16> {
    let listener =
        TcpListener::bind("127.0.0.1:0").context("bind 127.0.0.1:0 to pick ephemeral port")?;
    let port = listener
        .local_addr()
        .context("read local_addr for ephemeral port")?
        .port();
    drop(listener);
    Ok(port)
}

/// Build the minimum-footprint cluster config for an e2e test. Port
/// numbers are reserved via `pick_ephemeral_port` rather than fixed
/// constants so parallel runs (across processes) don't trip over each
/// other on host ports.
fn build_config(name: &str) -> Result<K3dClusterConfig> {
    // We avoid touching the host's /etc/hosts and routing tables —
    // e2e tests must be a no-op on the developer's machine outside the
    // cluster they create.
    let network = K3dNetworkConfig {
        configure_routes: false,
        configure_dns: false,
        patch_etc_hosts: false,
        // Defaults are fine for pod/service CIDRs; we never expose them.
        pod_cidr: "10.42.0.0/16".to_string(),
        service_cidr: "10.43.0.0/16".to_string(),
        // Bind only to 127.0.0.1 — the API server doesn't need to be
        // reachable from anything except local kubectl. This also
        // means concurrent clusters on the same host don't collide on
        // 0.0.0.0:<port> the way the default config would.
        api_bind_address: "127.0.0.1".to_string(),
        disable_iptables_routing: true,
    };

    // Reserve ephemeral ports up front. `K3dClusterConfig`'s port
    // fields are `u16` (not Option), and the underlying create_cluster
    // path always emits `-p <port>:<port>@loadbalancer` for both
    // http/https plus a registry health-check on localhost:<port>/v2/
    // — so we cannot pass 0 anywhere here, even though we'd love k3d
    // to auto-pick. Picking these here at least keeps the harness
    // from hardcoding well-known host ports.
    let api_port = pick_ephemeral_port()?;
    let http_port = pick_ephemeral_port()?;
    let https_port = pick_ephemeral_port()?;
    let registry_port = pick_ephemeral_port()?;

    Ok(K3dClusterConfig {
        name: name.to_string(),
        api_port,
        http_port,
        https_port,
        servers: 1,
        agents: 0,
        k3s_version: "v1.31.2-k3s1".to_string(),
        registry: K3dRegistryConfig {
            enabled: true,
            // Cluster-scoped registry name so `delete_cluster` can
            // also drop the registry (see manager.rs:466-475 in
            // k8s-manager — if the name contains the cluster name,
            // the registry is treated as cluster-scoped).
            name: format!("{}-registry", name),
            port: registry_port,
        },
        tls: K3dTlsConfig {
            // We don't need TLS for e2e probes; certificates are a
            // multi-second cost we can skip.
            enabled: false,
            ..K3dTlsConfig::default()
        },
        network,
        dev_mode: None::<K3dDevModeConfig>,
        timeouts: K3dTimeoutConfig::default(),
    })
}

/// Create a fresh per-test k3d cluster scoped to `chart`. The returned
/// [`ClusterHandle`] owns no resources directly — wrap it in a
/// [`ClusterGuard`] for RAII teardown.
///
/// The cluster name format is `firestream-e2e-<chart>-<rand>` where
/// `<rand>` is 6 lowercase alphanumeric chars. This is the exact
/// string passed to `k3d cluster create`, so it's also what
/// `k3d cluster list` and `docker ps --filter "name=k3d-<…>"` will
/// show after creation.
///
/// `K3dClusterManager::create_cluster` is async; we wrap it in a
/// current-thread tokio runtime so this crate's public API stays
/// synchronous (matching Phase 1's convention).
pub fn create_cluster(chart: &str) -> Result<ClusterHandle> {
    let name = format!("firestream-e2e-{}-{}", chart, random_suffix());
    let config = build_config(&name)?;
    let registry_name = format!("k3d-{}-registry", &name);
    let manager = K3dClusterManager::new(config);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build tokio current-thread runtime for cluster create")?;

    rt.block_on(manager.create_cluster())
        .with_context(|| format!("k3d cluster create {}", name))?;

    // k3d-the-binary writes a per-cluster kubeconfig we can fetch with
    // `k3d kubeconfig get <cluster>`. Stash it under temp_dir so it
    // doesn't leak into the user's $HOME/.kube/config; we never call
    // `merge` so we never touch any global config.
    let kubeconfig_path = std::env::temp_dir().join(format!("{}.kubeconfig", name));
    write_kubeconfig(&name, &kubeconfig_path).with_context(|| {
        format!(
            "writing kubeconfig for {} to {}",
            name,
            kubeconfig_path.display()
        )
    })?;

    Ok(ClusterHandle {
        name,
        kubeconfig: kubeconfig_path,
        registry: registry_name,
    })
}

/// Run `k3d kubeconfig get <name>` and persist stdout to `path`. We
/// can't use `K3dClusterManager::reconnect_cluster` because that
/// writes to the default location and mutates `$HOME/.kube/config`.
fn write_kubeconfig(cluster: &str, path: &std::path::Path) -> Result<()> {
    let out = Command::new("k3d")
        .args(["kubeconfig", "get", cluster])
        .stdin(Stdio::null())
        .output()
        .context("spawn k3d kubeconfig get")?;
    if !out.status.success() {
        bail!(
            "k3d kubeconfig get {} failed: status={} stderr={}",
            cluster,
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    fs::write(path, &out.stdout)
        .with_context(|| format!("write kubeconfig to {}", path.display()))?;
    Ok(())
}

/// RAII guard that deletes the cluster (and best-effort cleans up any
/// leftover containers) on Drop.
///
/// Skipped entirely when `keep` is true (`FIRESTREAM_E2E_K8S_KEEP=1`);
/// otherwise spawns `k3d cluster delete <name>` under a 60s budget. On
/// timeout / failure, falls back to `docker rm -f` over any container
/// matching `k3d-<name>-*`. The drop path is panic-safe: nothing
/// inside it unwraps a poisoned lock or `.expect()`s an env var.
pub struct ClusterGuard {
    handle: ClusterHandle,
    keep: bool,
}

impl ClusterGuard {
    /// Construct a new guard. Use `keep = env_keep_k8s()` from the
    /// harness to honor the contract env var.
    pub fn arm(handle: ClusterHandle, keep: bool) -> Self {
        Self { handle, keep }
    }

    /// Borrow the underlying handle (for tests that want to inspect
    /// the cluster without consuming the guard).
    pub fn handle(&self) -> &ClusterHandle {
        &self.handle
    }
}

impl Drop for ClusterGuard {
    fn drop(&mut self) {
        if self.keep {
            eprintln!(
                "[e2e-k8s:{}] KEEP=1 — leaving cluster up. \
                 Clean up manually with `k3d cluster delete {}` (and \
                 `k3d registry delete k3d-{}-registry`).",
                self.handle.name, self.handle.name, self.handle.name
            );
            return;
        }

        // Best-effort kubeconfig cleanup. Don't fail the drop if this
        // can't delete (e.g. someone moved the file).
        let _ = fs::remove_file(&self.handle.kubeconfig);

        // Primary path: k3d cluster delete <name> under a 60s budget.
        let mut primary_ok = false;
        let primary = Command::new("k3d")
            .args(["cluster", "delete", &self.handle.name])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match primary {
            Ok(mut child) => match wait_with_budget(&mut child, Duration::from_secs(60)) {
                Ok(()) => {
                    primary_ok = true;
                }
                Err(()) => {
                    warn!(
                        cluster = %self.handle.name,
                        "k3d cluster delete exceeded 60s budget; killing and falling back to docker rm"
                    );
                    let _ = child.kill();
                    let _ = child.wait();
                }
            },
            Err(e) => {
                warn!(
                    cluster = %self.handle.name,
                    err = %e,
                    "spawning k3d cluster delete failed; falling back to docker rm"
                );
            }
        }

        // Always also drop the cluster-scoped registry. `k3d cluster
        // delete` does NOT remove the registry container k3d created
        // for the cluster (the underlying K3dClusterManager::delete_cluster
        // does, but we don't go through it here because it's async).
        // The registry name is whatever we passed during create —
        // `<cluster>-registry`. Pass it under both the bare and
        // `k3d-`-prefixed form because k3d accepts either.
        let registry_short = format!("{}-registry", self.handle.name);
        for candidate in &[registry_short.as_str(), self.handle.registry.as_str()] {
            let _ = Command::new("k3d")
                .args(["registry", "delete", candidate])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        if primary_ok {
            return;
        }

        // Fallback: docker rm -f every container whose name starts with
        // `k3d-<cluster>-`. Covers server / agent / loadbalancer / registry.
        let prefix = format!("k3d-{}-", self.handle.name);
        let filter = format!("name={}", prefix);
        let list = Command::new("docker")
            .args(["ps", "-aq", "--filter", &filter])
            .stdin(Stdio::null())
            .output();
        if let Ok(out) = list {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let ids: Vec<&str> = stdout.split_whitespace().collect();
                if !ids.is_empty() {
                    let mut cmd = Command::new("docker");
                    cmd.arg("rm").arg("-f");
                    for id in &ids {
                        cmd.arg(id);
                    }
                    let _ = cmd
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
            }
        }
    }
}

/// Convenience: type-erased Box<dyn FnOnce()> shaped error for fall-back
/// callers that want to wrap a result with a cluster name. Currently
/// unused but reserved so Phase 3 doesn't have to redefine it.
#[allow(dead_code)]
pub(crate) fn anyhow_with_cluster(name: &str, e: anyhow::Error) -> anyhow::Error {
    anyhow!("cluster={} error: {}", name, e)
}

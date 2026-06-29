//! Kubernetes (k3d) backend for `firestream app`.
//!
//! Drives the firestream-charts bundle through the helm lifecycle
//! (`crate::deploy::helm::deploy_chart_lifecycle`, exactly like
//! `cli::commands::execute_helm_command`) and probes readiness via the
//! shared k8s probe matrix (`firestream_e2e_core::k8s::for_chart`).
//!
//! # Kubeconfig resolution
//!
//! - `--ephemeral`: `create_cluster(spec.chart)` stands up a fresh k3d
//!   cluster and returns a real [`ClusterHandle`]. `up`/`test` LEAVE the
//!   cluster running — we arm a [`ClusterGuard`] with `keep=true` and
//!   `std::mem::forget` it so Drop never deletes. Only `down --ephemeral`
//!   tears the cluster down (guard with `keep=false`).
//! - host-k3s (non-ephemeral): resolve the ambient kubeconfig in order
//!   `$KUBECONFIG` → `~/.kube/config` → `/etc/rancher/k3s/k3s.yaml` (first
//!   that exists) and wrap it in a synthetic `ClusterHandle { name: "host",
//!   kubeconfig, registry: "" }`. The probe path reads only
//!   `handle.kubeconfig`, so the synthetic handle is sufficient.
//!
//! `health` recomputes the same kubeconfig resolution. NOTE: a standalone
//! `app health <app> --ephemeral` invocation finds no cluster (each run
//! creates a new random-named one); ephemeral health is only meaningful
//! within a single `up`/`test` process. host-k3s health works standalone.
//!
//! # Demo-data override (known limitation)
//!
//! The three demo apps (airflow/superset/odoo) carry their demo toggle
//! inside the chart's `extraEnvVars` array — the SAME array that holds the
//! firestream path-remap env vars. Helm merges arrays by REPLACEMENT, so
//! injecting demo via an extra `-f` values file or `--set extraEnvVars[..]`
//! would clobber the path remaps and break the deploy (CLAUDE.md gotcha:
//! "Path remap (extraEnvVars)"). We therefore do NOT override demo on k8s;
//! we warn and proceed with the container-baked default. Forcing a demo
//! value on k8s is a follow-up that needs the flake to expose a dedicated
//! merge-safe values key (RFC).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::demo;
use super::registry::AppSpec;
use super::AppOpts;
use crate::core::{FirestreamError, Result};

use firestream_e2e_core::k8s::{
    create_cluster, for_chart, preload_images, wait_pods_ready, ClusterGuard, ClusterHandle,
    ImportTarget, K8sCtx,
};
use firestream_e2e_core::retry::retry_until_sync;

/// Resolve the effective namespace for an app: explicit override wins,
/// else the spec's default.
pub fn effective_namespace(spec: &AppSpec, opts: &AppOpts) -> String {
    opts.namespace
        .clone()
        .unwrap_or_else(|| spec.k8s_namespace.to_string())
}

/// Resolve the ambient kubeconfig for the host-k3s path.
///
/// Order: `$KUBECONFIG` → `~/.kube/config` → `/etc/rancher/k3s/k3s.yaml`.
/// Returns the first that exists; errors if none do.
fn ambient_kubeconfig() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("KUBECONFIG") {
        // KUBECONFIG may be a `:`-separated list; take the first entry.
        if let Some(first) = p.split(':').next() {
            let pb = PathBuf::from(first);
            if pb.exists() {
                return Ok(pb);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let kube = home.join(".kube").join("config");
        if kube.exists() {
            return Ok(kube);
        }
    }
    let k3s = PathBuf::from("/etc/rancher/k3s/k3s.yaml");
    if k3s.exists() {
        return Ok(k3s);
    }
    Err(FirestreamError::ConfigError(
        "no kubeconfig found: set $KUBECONFIG, or have ~/.kube/config or /etc/rancher/k3s/k3s.yaml"
            .to_string(),
    ))
}

/// Build a synthetic host-k3s handle around the ambient kubeconfig.
fn host_handle() -> Result<ClusterHandle> {
    Ok(ClusterHandle {
        name: "host".to_string(),
        kubeconfig: ambient_kubeconfig()?,
        registry: String::new(),
    })
}

/// Resolve the chart bundle dir from opts (threaded from `cli.charts_dir`),
/// falling back to the env/default resolution.
fn charts_dir(opts: &AppOpts) -> PathBuf {
    opts.charts_dir
        .clone()
        .unwrap_or_else(crate::deploy::helm_lifecycle::from_manifest::charts_dir)
}

/// Open the chart bundle and return the app's raw `ChartManifest`. Used by
/// the image-preload step, which needs the manifest's `images` map (the
/// `ChartInfo` returned by [`resolve_chart_info`] drops it).
fn open_manifest(spec: &AppSpec, opts: &AppOpts) -> Result<firestream_charts::ChartManifest> {
    use firestream_charts::Charts;

    let dir = charts_dir(opts);
    let charts = Charts::open(&dir).map_err(|e| {
        FirestreamError::ConfigError(format!(
            "Failed to open chart bundle at {:?}: {}. Set FIRESTREAM_CHARTS_DIR or pass --charts-dir.",
            dir, e
        ))
    })?;
    charts.get(spec.chart).map_err(|e| {
        FirestreamError::ConfigError(format!("Chart `{}` not found in {:?}: {}", spec.chart, dir, e))
    })
}

/// Build the `ChartInfo` for an app from the bundle, applying the namespace
/// override. Returns `(ChartInfo, release_name, namespace)`.
fn resolve_chart_info(
    spec: &AppSpec,
    opts: &AppOpts,
) -> Result<(crate::deploy::helm_lifecycle::ChartInfo, String, String)> {
    use crate::deploy::helm_lifecycle::chart_info_from_manifest;
    use firestream_charts::Charts;

    let dir = charts_dir(opts);
    let charts = Charts::open(&dir).map_err(|e| {
        FirestreamError::ConfigError(format!(
            "Failed to open chart bundle at {:?}: {}. Set FIRESTREAM_CHARTS_DIR or pass --charts-dir.",
            dir, e
        ))
    })?;
    let manifest = charts.get(spec.chart).map_err(|e| {
        FirestreamError::ConfigError(format!(
            "Chart `{}` not found in {:?}: {}",
            spec.chart, dir, e
        ))
    })?;

    let mut info = chart_info_from_manifest(&manifest);
    let namespace = effective_namespace(spec, opts);
    info.custom_namespace = Some(namespace.clone());
    // Release name = ChartInfo.name (manifest name), which is what the
    // helm executor uses for `helm upgrade/uninstall <name>` and what the
    // bitnami `app.kubernetes.io/instance` label carries.
    let release = info.name.clone();
    Ok((info, release, namespace))
}

/// Deploy the app's chart and (optionally) wait for readiness + health.
/// LEAVES the deployment (and ephemeral cluster) running.
pub async fn up(spec: &AppSpec, opts: &AppOpts) -> Result<()> {
    use crate::deploy::helm::deploy_chart_lifecycle;
    use crate::deploy::helm_lifecycle::CommonChart;

    // Demo override is unsupported on k8s (see module docs). Warn loudly.
    if demo::is_demo_noop(spec, opts.demo) {
        eprintln!(
            "[app:{}] note: --demo/--no-demo is a no-op for this app (no demo-data toggle)",
            spec.name
        );
    } else if opts.demo.is_some() && spec.supports_demo() {
        eprintln!(
            "[app:{}] WARNING: k8s backend cannot override demo-data without clobbering the \
             chart's extraEnvVars path-remaps; deploying with the container-baked default \
             instead. (docker backend supports the override.)",
            spec.name
        );
    }

    // Resolve kubeconfig: ephemeral cluster or ambient host-k3s.
    let handle = if opts.ephemeral {
        println!(
            "[app:{}] up via k8s: creating ephemeral k3d cluster for chart `{}`",
            spec.name, spec.chart
        );
        let h = create_cluster(spec.chart).map_err(|e| {
            FirestreamError::GeneralError(format!("create ephemeral k3d cluster: {}", e))
        })?;
        // LEAVE the cluster running: arm a keep=true guard and forget it so
        // Drop never deletes. (up/test must not tear down.)
        let guard = ClusterGuard::arm(h.clone(), /*keep=*/ true);
        std::mem::forget(guard);
        println!(
            "[app:{}] ephemeral cluster `{}` left running (kubeconfig {:?})",
            spec.name,
            h.name,
            h.kubeconfig
        );
        h
    } else {
        host_handle()?
    };

    // Preload the chart's firestream-* images into the target cluster's
    // containerd BEFORE the helm deploy. The charts reference registry-less
    // `firestream-<app>:<tag>` with `pullPolicy: IfNotPresent`; without a
    // side-load the pods ImagePullBackOff. Ephemeral → k3d import; host →
    // host-k3s (docker save → ctr import). Gated by `--no-preload` /
    // `FIRESTREAM_APP_PRELOAD=0` (resolved into `opts.preload`).
    if opts.preload {
        let target = if opts.ephemeral {
            ImportTarget::K3d {
                cluster: handle.name.clone(),
            }
        } else {
            ImportTarget::HostK3s
        };
        let manifest = open_manifest(spec, opts)?;
        println!(
            "[app:{}] preloading firestream-* images into {} (target={:?})",
            spec.name,
            if opts.ephemeral { "ephemeral k3d cluster" } else { "host k3s containerd" },
            target
        );
        preload_images(&target, &manifest).map_err(|e| {
            FirestreamError::GeneralError(format!(
                "preload images for `{}` failed: {:#}. The helm deploy would \
                 ImagePullBackOff without these images. For host k3s the default \
                 importer is `ctr -a /run/k3s/containerd/containerd.sock -n k8s.io \
                 images import` and needs access to that socket (k3s group or root); \
                 set FIRESTREAM_K8S_IMPORT_CMD to a privileged wrapper if needed, or \
                 pass --no-preload to skip.",
                spec.chart, e
            ))
        })?;
    } else {
        println!(
            "[app:{}] preload disabled (--no-preload / FIRESTREAM_APP_PRELOAD=0); \
             pods will ImagePullBackOff unless images are already in the cluster",
            spec.name
        );
    }

    let (info, release, namespace) = resolve_chart_info(spec, opts)?;

    println!(
        "[app:{}] deploying chart `{}` (ns={}, release={}) via helm lifecycle",
        spec.name, info.chart, namespace, release
    );
    let common = CommonChart::new(info);
    deploy_chart_lifecycle(Box::new(common), Some(&handle.kubeconfig)).await?;

    if opts.wait {
        let deadline = Instant::now() + Duration::from_secs(opts.timeout_secs.max(1));
        println!("[app:{}] waiting for pods Ready (ns={})", spec.name, namespace);
        wait_pods_ready(&handle.kubeconfig, &release, &namespace, deadline).map_err(|e| {
            FirestreamError::GeneralError(format!("wait_pods_ready: {}", e))
        })?;
        run_probes(spec, &handle, &release, &namespace, opts).await?;
    }
    Ok(())
}

/// Run the per-chart probe matrix once. Shared by `up --wait` and `health`.
///
/// The probes are synchronous and some (`HttpProbeOverPortForward`) drive a
/// `reqwest::blocking` client internally, which panics if its runtime is
/// dropped on a tokio worker thread. We therefore run the whole loop inside
/// `spawn_blocking`.
async fn run_probes(
    spec: &AppSpec,
    handle: &ClusterHandle,
    release: &str,
    namespace: &str,
    opts: &AppOpts,
) -> Result<()> {
    let chart = spec.chart.to_string();
    let app_name = spec.name.to_string();
    let handle = handle.clone();
    let release = release.to_string();
    let namespace = namespace.to_string();
    let timeout_secs = opts.timeout_secs.max(1);

    tokio::task::spawn_blocking(move || {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        let ctx = K8sCtx {
            handle,
            namespace,
            release,
            deadline,
        };
        let probes = for_chart(&chart, &ctx).map_err(|e| {
            FirestreamError::GeneralError(format!("build probe chain for `{}`: {}", chart, e))
        })?;

        let mut all_ok = true;
        for probe in probes.iter() {
            let pname = probe.name();
            match retry_until_sync(&deadline, || probe.run(&ctx)) {
                Ok(()) => println!("[app:{}] probe {}: PASS", app_name, pname),
                Err(e) => {
                    println!("[app:{}] probe {}: FAIL ({})", app_name, pname, e);
                    all_ok = false;
                }
            }
        }
        if all_ok {
            Ok(())
        } else {
            Err(FirestreamError::GeneralError(format!(
                "[app:{}] one or more k8s probes failed",
                app_name
            )))
        }
    })
    .await
    .map_err(|e| FirestreamError::GeneralError(format!("probe task join error: {}", e)))?
}

/// Probe the app's readiness via the shared probe matrix. Recomputes the
/// kubeconfig (host-k3s) — ephemeral health only works inside an `up`/`test`
/// process (see module docs).
pub async fn health(spec: &AppSpec, opts: &AppOpts) -> Result<()> {
    if opts.ephemeral {
        eprintln!(
            "[app:{}] WARNING: standalone `health --ephemeral` cannot locate the per-run cluster; \
             use it within `app up`/`app test`. Falling back to host kubeconfig.",
            spec.name
        );
    }
    let handle = host_handle()?;
    // We don't re-open the bundle for health (no deploy), but we need the
    // release name + namespace. Release name = chart name unless the
    // manifest set a custom one; resolve from the bundle when available,
    // else fall back to chart name.
    let (release, namespace) = match resolve_chart_info(spec, opts) {
        Ok((_, release, ns)) => (release, ns),
        Err(_) => (spec.chart.to_string(), effective_namespace(spec, opts)),
    };
    println!(
        "[app:{}] health via k8s (ns={}, release={})",
        spec.name, namespace, release
    );
    run_probes(spec, &handle, &release, &namespace, opts).await
}

/// Uninstall the app's chart (and the ephemeral cluster, if any).
pub async fn down(spec: &AppSpec, opts: &AppOpts) -> Result<()> {
    use crate::deploy::helm_lifecycle::executor::helm_uninstall;

    if opts.ephemeral {
        // For ephemeral down we'd normally tear the cluster down, but to
        // delete it we'd need the random cluster name from the original
        // `up` (a fresh process doesn't have it). Instruct the user and
        // fall through to a host-kubeconfig uninstall.
        eprintln!(
            "[app:{}] note: `down --ephemeral` cannot reconstruct the random k3d cluster name \
             created by a prior `up`; delete it manually with `k3d cluster list` + \
             `k3d cluster delete <name>`. Proceeding to uninstall the release against the \
             host kubeconfig (no-op if it lives in the ephemeral cluster).",
            spec.name
        );
    }
    let handle = host_handle()?;

    let (info, release, namespace) = match resolve_chart_info(spec, opts) {
        Ok(t) => t,
        Err(e) => {
            // Without the bundle we can still uninstall by name: build a
            // minimal ChartInfo carrying name + namespace.
            eprintln!(
                "[app:{}] chart bundle unavailable ({}); uninstalling by name",
                spec.name, e
            );
            let ns = effective_namespace(spec, opts);
            let info = crate::deploy::helm_lifecycle::ChartInfo {
                name: spec.chart.to_string(),
                custom_namespace: Some(ns.clone()),
                ..Default::default()
            };
            (info, spec.chart.to_string(), ns)
        }
    };

    println!(
        "[app:{}] down via k8s: helm uninstall {} -n {}",
        spec.name, release, namespace
    );
    let envs: Vec<(String, String)> = vec![];
    helm_uninstall(&handle.kubeconfig, &envs, &info).await?;
    Ok(())
}

/// Best-effort per-app status line: `kubectl get pods -l instance=<release>`.
pub fn status_line(spec: &AppSpec, opts: &AppOpts) {
    let handle = match host_handle() {
        Ok(h) => h,
        Err(_) => {
            println!("[app:{}] k8s: <no kubeconfig>", spec.name);
            return;
        }
    };
    let (release, namespace) = match resolve_chart_info(spec, opts) {
        Ok((_, r, n)) => (r, n),
        Err(_) => (spec.chart.to_string(), effective_namespace(spec, opts)),
    };
    let selector = format!("app.kubernetes.io/instance={}", release);
    let out = std::process::Command::new("kubectl")
        .arg("--kubeconfig")
        .arg(&handle.kubeconfig)
        .args([
            "get",
            "pods",
            "-l",
            &selector,
            "-n",
            &namespace,
            "--no-headers",
        ])
        .stdin(std::process::Stdio::null())
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let body = String::from_utf8_lossy(&o.stdout);
            let trimmed = body.trim();
            if trimmed.is_empty() {
                println!("[app:{}] k8s (ns {}): no pods", spec.name, namespace);
            } else {
                println!("[app:{}] k8s (ns {}):", spec.name, namespace);
                for line in trimmed.lines() {
                    println!("    {}", line);
                }
            }
        }
        _ => println!("[app:{}] k8s (ns {}): <get pods failed>", spec.name, namespace),
    }
}

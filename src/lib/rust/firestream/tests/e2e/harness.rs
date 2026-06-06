// E2E harness orchestration (docker-compose backend): per-stack lifecycle,
// skip/filter gates, serialisation, port discovery via `nix eval --json`,
// retry-with-deadline, and the StackGuard whose Drop runs `nix run .#<name>-down`
// under a 60s wall-clock timeout (audit correction 6b).
//
// As of Phase 1 of the k3d/helm e2e plan, the transport-agnostic primitives
// (Probe trait, retry loop, env accessors, drop-budget wait helper, Exec
// trait) have moved to `firestream-e2e-core`. This file is the
// docker-specific harness that wraps them.
//
// Probes are SYNCHRONOUS (see `probes.rs`). No tokio runtime is created in
// run_one. This eliminates the "runtime dropped in unwind" hazard the audit
// flagged: there is nothing async to drop, ever.
//
// ------------------------------------------------------------------
// Environment-variable contract (Phase 4)
// ------------------------------------------------------------------
//
// | Var                            | Meaning                                                                           | Default          |
// |--------------------------------|-----------------------------------------------------------------------------------|------------------|
// | `FIRESTREAM_E2E_STACKS`        | CSV subset of canonical stack names; `all` ⇒ every key                            | canonical set    |
// | `FIRESTREAM_E2E_KEEP`          | `1` ⇒ skip teardown; leave stack(s) running                                       | unset            |
// | `FIRESTREAM_E2E_STRICT`        | `1` ⇒ skip-gate failure (no nix/docker) becomes a hard panic                      | unset            |
// | `FIRESTREAM_E2E_TIMEOUT_SECS`  | Per-stack readiness deadline (does NOT bound build time)                          | `300`            |
// | `FIRESTREAM_E2E_PREBUILD`      | `1` ⇒ parallel `nix run .#<image>-image -- --load` outside the global lock        | unset            |
// | `FIRESTREAM_E2E_HEALTHD`       | `0` ⇒ skip the unified HealthEndpoint chain; use per-protocol probes (Phase 1)    | `1` (use it)     |
// | `FIRESTREAM_E2E_HTTP`          | `0` ⇒ skip every HTTP-based probe (HealthEndpoint, HttpReady, SparkWithWorkers)   | `1`              |
//
// `STACKS`, `KEEP`, `STRICT`, `TIMEOUT_SECS` are read via
// `firestream_e2e_core::env`; `PREBUILD`, `HEALTHD`, `HTTP` remain
// docker-harness-local because the next backend (k8s, Phase 2+) uses
// different mechanisms for image preload and health-endpoint discovery.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use firestream_e2e_core::env::{
    docker_daemon_check, env_keep, env_strict, env_timeout_secs, selected, should_skip,
};
use firestream_e2e_core::guard::wait_with_budget;
use firestream_e2e_core::retry::retry_until_sync;

use crate::probes::{self, Probe, ProbeCtx};
use crate::stacks;

// ----- Global serialisation -----

/// All canonical e2e stacks share a single global lock so they never race for
/// host ports or for the docker daemon's image-load step. Correctness no
/// longer depends on `--test-threads=1`, but the Makefile target still passes
/// it so cargo's per-test output isn't interleaved.
fn e2e_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Optional prebuild: runs every selected stack's image-build list under
/// `nix run .#<image>-image -- --load` *outside* the global lock, populating
/// the per-arch Nix store volume so the locked `up` phase is fast.
///
/// Strictly an opt-in optimisation for warm-cache reruns. Default off — the
/// audited mutex semantics keep the locked-up phase race-free either way.
fn prebuild_once_if_enabled(stack: &str, build_list: &[String]) {
    if std::env::var("FIRESTREAM_E2E_PREBUILD").ok().as_deref() != Some("1") {
        return;
    }
    static ONCE: OnceLock<Mutex<std::collections::BTreeSet<String>>> = OnceLock::new();
    let lock = ONCE.get_or_init(|| Mutex::new(std::collections::BTreeSet::new()));
    let mut seen = lock.lock().unwrap_or_else(|p| p.into_inner());
    for image in build_list {
        if seen.contains(image) {
            continue;
        }
        eprintln!("[e2e:{}] PREBUILD: nix run .#{}-image -- --load", stack, image);
        let status = Command::new("nix")
            .args(["run", &format!(".#{}-image", image), "--", "--load"])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        match status {
            Ok(s) if s.success() => {
                seen.insert(image.clone());
            }
            Ok(s) => {
                eprintln!("[e2e:{}] PREBUILD image {}: nix run exited {}", stack, image, s);
            }
            Err(e) => {
                eprintln!("[e2e:{}] PREBUILD image {}: nix run spawn failed: {}", stack, image, e);
            }
        }
    }
}

// ----- Skip / strict gates -----

/// Returns Some(reason) if the harness cannot run on this host, else None.
/// `FIRESTREAM_E2E_STRICT=1` upgrades this to a hard panic in `run_one`.
///
/// Two-stage gate:
///   1. `should_skip(&["nix", "docker"])` — both binaries on PATH.
///   2. `docker_daemon_check()` — the daemon answers.
/// Stage 2 lives in core because the docker harness is the only current
/// consumer but the implementation has no docker-specific knowledge beyond
/// "shell out to `docker info`".
fn skip_reason() -> Option<String> {
    if let Some(r) = should_skip(&["nix", "docker"]) {
        return Some(r);
    }
    docker_daemon_check()
}

// ----- Discovery via `nix eval --json` (audit correction 5) -----

/// Resolved per-stack info read from Nix.
struct StackInfo {
    /// Path to the rendered docker-compose.yml (`<store>/docker-compose.yml`).
    compose_file: PathBuf,
    /// docker compose project name.
    project: String,
    /// Host ports published (post-offset).
    host_ports: Vec<u16>,
    /// Post-offset host port for firestream-healthd (Phase 3+). `None` when
    /// the stack's container has `health.enable = false`.
    health_host_port: Option<u16>,
    /// Image build order — this stack + its declared `compose.dependencies`.
    build_list: Vec<String>,
}

fn nix_eval_json(attr: &str) -> anyhow::Result<serde_json::Value> {
    let out = Command::new("nix")
        .args(["eval", "--json", attr])
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::anyhow!("spawn nix eval {}: {}", attr, e))?;
    if !out.status.success() {
        anyhow::bail!("nix eval {} exited {}", attr, out.status);
    }
    serde_json::from_slice(&out.stdout).map_err(|e| anyhow::anyhow!("parse nix eval {}: {}", attr, e))
}

fn nix_build_compose(stack: &str) -> anyhow::Result<PathBuf> {
    let out = Command::new("nix")
        .args(["build", "--no-link", "--print-out-paths", &format!(".#{}-compose", stack)])
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::anyhow!("spawn nix build .#{}-compose: {}", stack, e))?;
    if !out.status.success() {
        anyhow::bail!("nix build .#{}-compose exited {}", stack, out.status);
    }
    let store_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if store_path.is_empty() {
        anyhow::bail!("nix build .#{}-compose produced no output path", stack);
    }
    Ok(PathBuf::from(store_path).join("docker-compose.yml"))
}

fn resolve_stack(stack: &str) -> anyhow::Result<StackInfo> {
    // Ports come from option evaluation (no rendered-YAML parsing).
    let ports_v = nix_eval_json(&format!(".#{}-compose.passthru.hostPorts", stack))?;
    let host_ports: Vec<u16> = ports_v
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("hostPorts is not an array"))?
        .iter()
        .filter_map(|v| v.as_u64())
        .filter_map(|n| u16::try_from(n).ok())
        .collect();

    let project_v = nix_eval_json(&format!(".#{}-compose.passthru.projectName", stack))?;
    let project = project_v
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("projectName is not a string"))?
        .to_string();

    let build_v = nix_eval_json(&format!(".#{}-compose.passthru.buildList", stack))?;
    let build_list: Vec<String> = build_v
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("buildList is not an array"))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    // Phase 3: `healthHostPort` is `null` for stacks with health disabled,
    // or `9180 + offset` for those that opted in. The harness reads it once
    // here so per-probe code never recomputes the offset itself.
    let health_host_port = match nix_eval_json(&format!(
        ".#{}-compose.passthru.healthHostPort",
        stack
    )) {
        Ok(v) if v.is_null() => None,
        Ok(v) => v.as_u64().and_then(|n| u16::try_from(n).ok()),
        // A failure here means the attr is missing (older flake checkout);
        // treat as "not configured" rather than aborting the test.
        Err(_) => None,
    };

    // Build the compose store path so docker compose -f points at a fixed file.
    let compose_file = nix_build_compose(stack)?;

    Ok(StackInfo {
        compose_file,
        project,
        host_ports,
        health_host_port,
        build_list,
    })
}

// ----- nix run wrappers -----

fn nix_run_up(stack: &str) -> anyhow::Result<()> {
    let status = Command::new("nix")
        .args(["run", &format!(".#{}-up", stack)])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("spawn nix run .#{}-up: {}", stack, e))?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("nix run .#{}-up exited {}", stack, status)
    }
}

/// Idempotent pre-clean. Errors are logged, not propagated. Passes `-v` so
/// docker compose also removes the project's volumes — e2e test isolation
/// requires fresh state for every iteration (otherwise re-running the same
/// stack reuses a prior data volume, which some entrypoints can't re-init).
fn nix_run_down(stack: &str) {
    let _ = Command::new("nix")
        .args(["run", &format!(".#{}-down", stack), "--", "-v"])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
}

// ----- StackGuard: 60s teardown budget, fallback to `compose down -v` (audit 6b) -----

struct StackGuard {
    name: String,
    compose_file: PathBuf,
    project: String,
    /// Read at construction so Drop is independent of env mutation later.
    keep: bool,
}

impl StackGuard {
    fn new(name: &str, compose_file: PathBuf, project: String, keep: bool) -> Self {
        Self {
            name: name.to_string(),
            compose_file,
            project,
            keep,
        }
    }
}

const DROP_BUDGET: Duration = Duration::from_secs(60);

impl Drop for StackGuard {
    fn drop(&mut self) {
        if self.keep {
            eprintln!(
                "[e2e:{}] KEEP=1 — leaving stack running (docker compose -p {} ps to inspect)",
                self.name, self.project
            );
            return;
        }

        eprintln!("[e2e:{}] teardown: nix run .#{}-down -- -v (≤60s)", self.name, self.name);
        let mut child = match Command::new("nix")
            .args(["run", &format!(".#{}-down", self.name), "--", "-v"])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[e2e:{}] teardown: nix run spawn failed ({}); falling back to docker compose down -v",
                    self.name, e
                );
                self.fallback_compose_down();
                return;
            }
        };
        match wait_with_budget(&mut child, DROP_BUDGET) {
            Ok(()) => {
                eprintln!("[e2e:{}] teardown: nix run completed", self.name);
            }
            Err(()) => {
                eprintln!(
                    "[e2e:{}] teardown: nix run wedged past 60s; killing and falling back",
                    self.name
                );
                let _ = child.kill();
                let _ = child.wait();
                self.fallback_compose_down();
            }
        }
    }
}

impl StackGuard {
    fn fallback_compose_down(&self) {
        eprintln!(
            "[e2e:{}] teardown: docker compose -f {} -p {} down -v (≤60s)",
            self.name,
            self.compose_file.display(),
            self.project
        );
        let mut child = match Command::new("docker")
            .args(["compose", "-f"])
            .arg(&self.compose_file)
            .args(["-p", &self.project, "down", "-v"])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[e2e:{}] teardown: docker compose spawn failed ({}); giving up",
                    self.name, e
                );
                return;
            }
        };
        match wait_with_budget(&mut child, DROP_BUDGET) {
            Ok(()) => {
                eprintln!("[e2e:{}] teardown: docker compose down completed", self.name);
            }
            Err(()) => {
                let _ = child.kill();
                let _ = child.wait();
                eprintln!(
                    "[e2e:{}] WARNING: docker compose down -v wedged past 60s; giving up — \
                     manual cleanup likely required (`docker ps -a`, `docker volume ls`)",
                    self.name
                );
            }
        }
    }
}

// ----- Public entry: run_one -----

/// Drive one canonical stack: skip-gate → filter-gate → serialise → resolve
/// → pre-clean → up → arm guard → probe → drop guard (teardown).
pub fn run_one(name: &str) {
    // 1. Skip / strict gate.
    if let Some(reason) = skip_reason() {
        if env_strict() {
            panic!("[e2e:{}] STRICT: {}", name, reason);
        } else {
            eprintln!("[e2e:{}] SKIP: {}", name, reason);
            return;
        }
    }

    // 2. Filter gate. `selected()` is generic over the canonical-set
    //    predicate; the docker harness passes its own `is_canonical`.
    if !selected(name, stacks::is_canonical) {
        eprintln!("[e2e:{}] SKIP: not selected by FIRESTREAM_E2E_STACKS", name);
        return;
    }

    // 3. Serialise the entire lifecycle.
    let _serialize = e2e_lock().lock().unwrap_or_else(|p| p.into_inner());

    // 4. Discover ports/projectName/buildList from Nix (no YAML parsing).
    eprintln!("[e2e:{}] resolving via `nix eval --json` …", name);
    let info = match resolve_stack(name) {
        Ok(i) => i,
        Err(e) => panic!("[e2e:{}] resolve_stack failed: {:?}", name, e),
    };
    eprintln!(
        "[e2e:{}] resolved project={} compose={} host_ports={:?} health_host_port={:?} build_list={:?}",
        name,
        info.project,
        info.compose_file.display(),
        info.host_ports,
        info.health_host_port,
        info.build_list
    );

    // 4b. Optional prebuild (opt-in only; default off — see plan).
    prebuild_once_if_enabled(name, &info.build_list);

    // 5. Idempotent pre-clean.
    eprintln!("[e2e:{}] pre-clean: nix run .#{}-down", name, name);
    nix_run_down(name);

    // 6. Up + arm teardown.
    eprintln!("[e2e:{}] starting: nix run .#{}-up", name, name);
    if let Err(e) = nix_run_up(name) {
        panic!("[e2e:{}] nix run .#{}-up failed: {:?}", name, name, e);
    }
    let _guard = StackGuard::new(name, info.compose_file.clone(), info.project.clone(), env_keep());

    // 7. Probe loop — all probes are sync; no tokio runtime created (audit 6).
    let deadline = Instant::now() + Duration::from_secs(env_timeout_secs());
    let ctx = ProbeCtx {
        compose_file: info.compose_file.clone(),
        project: info.project.clone(),
        host_ports: info.host_ports.clone(),
        health_host_port: info.health_host_port,
        deadline,
    };
    let probe_chain: Vec<Box<dyn Probe>> = probes::for_stack(name, &ctx);
    if probe_chain.is_empty() {
        eprintln!(
            "[e2e:{}] WARN: empty probe chain (no host_ports?); treating as success",
            name
        );
        return;
    }
    for probe in probe_chain.iter() {
        eprintln!("[e2e:{}] probe: {}", name, probe.name());
        if let Err(e) = retry_until_sync(&deadline, || probe.run(&ctx)) {
            let failed = probe.name().to_string();
            probes::dump_diagnostics(&ctx.compose_file, &ctx.project);
            // StackGuard's Drop will run when this stack frame unwinds via panic.
            panic!("[e2e:{}] probe '{}' failed: {:?}", name, failed, e);
        }
    }

    eprintln!("[e2e:{}] all probes passed", name);
    // _guard drops here, running teardown unless KEEP=1.
}

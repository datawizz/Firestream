// Per-protocol readiness probes for canonical Firestream stacks (docker-
// compose backend).
//
// As of Phase 1 of the k3d/helm e2e plan, the transport-agnostic primitives
// (`Probe` trait, `Tcp`, `HttpReady`, `PgIsReady`, `RedisPing`,
// `KafkaApiVersions`) have moved to `firestream-e2e-core`. This file is now
// the **docker-specific** probe matrix:
//
//   * `DockerComposeExec` — `Exec` impl that shells out to
//     `docker compose -f <compose_file> -p <project> exec -T <service> <cmd…>`.
//   * `ProbeCtx`           — per-stack context populated from `nix eval --json`.
//   * `SparkWithWorkers`   — REST-JSON probe specific to the Spark master.
//   * `HealthEndpoint`     — `firestream-healthd` `/readyz` + `/sbom` probe.
//   * `for_stack(stack, ctx)` — the per-stack chain matrix the docker harness
//     consults.
//   * `dump_diagnostics`   — `docker compose ps` + `logs --tail 200` on failure.
//
// Audit correction 3: a blanket TCP-is-bound check is near-vacuous for several
// stacks (Kafka's listener binds before metadata is serviceable; a Spark
// master binds before workers register). Each probe below tests the
// protocol-level invariant that downstream consumers actually depend on.
//
// All probes are SYNCHRONOUS. The harness drives them inside a retry loop
// with a wall-clock deadline (see `firestream_e2e_core::retry::retry_until_sync`).

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};

use firestream_e2e_core::exec::Exec;
use firestream_e2e_core::probe::{
    HttpReady, KafkaApiVersions, PgIsReady, RedisPing, Tcp,
};

// ---------- DockerComposeExec: Exec impl for `docker compose exec -T` ----------

/// `Exec` impl that translates `target` into a compose service inside a fixed
/// `(compose_file, project)` pair and runs the requested command via
/// `docker compose -f <file> -p <project> exec -T <target> <args…>`.
///
/// One instance per stack — stash it in an `Arc` and hand it to every
/// exec-driven probe in the stack's chain.
pub struct DockerComposeExec {
    pub compose_file: PathBuf,
    pub project_name: String,
}

impl Exec for DockerComposeExec {
    fn exec(&self, target: &str, args: &[&str]) -> Result<Output> {
        let mut cmd = Command::new("docker");
        cmd.args(["compose", "-f"])
            .arg(&self.compose_file)
            .args(["-p", &self.project_name, "exec", "-T", target])
            .args(args)
            .stdin(Stdio::null());
        cmd.output()
            .with_context(|| format!("spawn docker compose exec {} {:?}", target, args))
    }
}

// ---------- ProbeCtx ----------

/// Per-probe context populated by the docker harness from `nix eval` data.
pub struct ProbeCtx {
    /// Path to the rendered docker-compose.yml (for `docker compose -f`).
    pub compose_file: PathBuf,
    /// docker compose project name (-p).
    pub project: String,
    /// Host ports actually published by this stack (post-offset).
    pub host_ports: Vec<u16>,
    /// Post-offset host port for the in-image firestream-healthd service
    /// (Phase 3+). `None` when the container has `health.enable = false`.
    /// Read directly from `passthru.healthHostPort` so the Rust harness
    /// never has to recompute the offset itself.
    pub health_host_port: Option<u16>,
    /// Wall-clock deadline for the entire readiness phase of this stack.
    /// Allow dead_code: kept for Phase 3+ probes that bound their own
    /// per-probe operations against the outer deadline.
    #[allow(dead_code)]
    pub deadline: Instant,
}

impl ProbeCtx {
    /// Construct a fresh `Arc<dyn Exec>` for the exec-driven probes in this
    /// stack's chain. Cheap — the result wraps a `DockerComposeExec`
    /// which holds only a path and a string.
    fn exec(&self) -> Arc<dyn Exec> {
        Arc::new(DockerComposeExec {
            compose_file: self.compose_file.clone(),
            project_name: self.project.clone(),
        })
    }
}

// ---------- Probe-trait adapter ----------
//
// `firestream_e2e_core::probe::Probe::run` takes `()` (transport-agnostic),
// but the docker harness's run loop calls `probe.run(&ctx)`. Wrap every core
// probe in a thin adapter so the for_stack() return type stays a
// `Vec<Box<dyn LocalProbe>>` and the harness code is unchanged.

/// Local probe trait — same shape as the original `probes::Probe`, takes the
/// docker `ProbeCtx`. Implemented for both:
///   * `CoreProbe<T>`           — adapter for transport-agnostic core probes
///   * `SparkWithWorkers`       — docker-specific (HTTP-based) probe
///   * `HealthEndpoint`         — docker-specific firestream-healthd probe
pub trait Probe_ {
    fn name(&self) -> &str;
    fn run(&self, ctx: &ProbeCtx) -> Result<()>;
}

// Re-export under the historical name so the harness imports stay untouched.
pub use Probe_ as Probe;

/// Adapter from `firestream_e2e_core::probe::Probe` (no ctx) to the harness's
/// local `Probe` trait (takes `&ProbeCtx`).
pub struct CoreProbe<T: firestream_e2e_core::probe::Probe> {
    inner: T,
}

impl<T: firestream_e2e_core::probe::Probe> CoreProbe<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: firestream_e2e_core::probe::Probe> Probe_ for CoreProbe<T> {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn run(&self, _ctx: &ProbeCtx) -> Result<()> {
        self.inner.run()
    }
}

/// Convenience: `Box<CoreProbe<T>> as Box<dyn Probe_>` in one call.
fn boxed<T: firestream_e2e_core::probe::Probe + 'static>(p: T) -> Box<dyn Probe_> {
    Box::new(CoreProbe::new(p))
}

// ---------- spark master + worker count (docker-specific) ----------

/// HTTP GET on master REST endpoint, parse JSON, require `aliveworkers >= min_workers`.
/// Reference: Spark master serves `/json/` returning `{ aliveworkers, ... }`.
pub struct SparkWithWorkers {
    pub master_url: String, // e.g. "http://127.0.0.1:8080"
    pub min_workers: u32,
}

impl Probe_ for SparkWithWorkers {
    fn name(&self) -> &str {
        "spark_with_workers"
    }
    fn run(&self, _ctx: &ProbeCtx) -> Result<()> {
        // Reach the UI first (proves the master process is up).
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .context("build reqwest blocking client")?;

        let json_url = format!("{}/json/", self.master_url.trim_end_matches('/'));
        let resp = client.get(&json_url).send().context("GET spark master json")?;
        let status = resp.status().as_u16();
        if !(status < 500) {
            bail!("spark master /json/ status {}", status);
        }
        // Parse via text() + serde_json so we don't pull the `json` feature
        // into reqwest (keeps the dev-dep closure minimal).
        let text = resp.text().context("read spark master json body")?;
        let body: serde_json::Value =
            serde_json::from_str(&text).context("parse spark master json")?;
        // Spark's REST JSON uses "aliveworkers" (lowercase, one word) historically.
        let alive = body
            .get("aliveworkers")
            .and_then(|v: &serde_json::Value| v.as_u64())
            .or_else(|| body.get("aliveWorkers").and_then(|v: &serde_json::Value| v.as_u64()))
            .ok_or_else(|| anyhow!("spark master json missing aliveworkers/aliveWorkers field"))?;
        if (alive as u32) >= self.min_workers {
            Ok(())
        } else {
            Err(anyhow!("spark aliveworkers={} < required {}", alive, self.min_workers))
        }
    }
}

// ---------- firestream-healthd /readyz + /sbom (docker-specific) ----------

/// Probe the in-image `firestream-healthd` service. Polls `<base_url>/readyz`
/// until 200, then makes a single `<base_url>/sbom?format=cyclonedx` request
/// and asserts the document is a JSON object containing a non-empty
/// `components` array. The /sbom check happens once on the first /readyz
/// success — the harness retry loop is only responsible for the readiness
/// portion; a /sbom failure escalates immediately because the SBOM is baked
/// at build time and retries cannot fix a malformed document.
pub struct HealthEndpoint {
    pub base_url: String,
}

impl Probe_ for HealthEndpoint {
    fn name(&self) -> &str {
        "health-endpoint"
    }
    fn run(&self, _ctx: &ProbeCtx) -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .context("build reqwest blocking client")?;

        // Step 1: /readyz must return 200.
        let readyz_url = format!("{}/readyz", self.base_url.trim_end_matches('/'));
        let resp = client
            .get(&readyz_url)
            .send()
            .with_context(|| format!("GET {}", readyz_url))?;
        let status = resp.status().as_u16();
        if status != 200 {
            bail!("GET {} status {} (want 200)", readyz_url, status);
        }

        // Step 2: /sbom?format=cyclonedx must parse as an object with a
        // non-empty `components` array.
        let sbom_url = format!(
            "{}/sbom?format=cyclonedx",
            self.base_url.trim_end_matches('/')
        );
        let sbom_resp = client
            .get(&sbom_url)
            .send()
            .with_context(|| format!("/sbom assertion: GET {}", sbom_url))?;
        let sbom_status = sbom_resp.status().as_u16();
        if sbom_status != 200 {
            bail!(
                "/sbom assertion: GET {} status {} (want 200)",
                sbom_url,
                sbom_status
            );
        }
        let body = sbom_resp
            .text()
            .with_context(|| format!("/sbom assertion: read body of {}", sbom_url))?;
        let doc: serde_json::Value = serde_json::from_str(&body)
            .with_context(|| format!("/sbom assertion: parse JSON from {}", sbom_url))?;
        let obj = doc
            .as_object()
            .ok_or_else(|| anyhow!("/sbom assertion: top-level JSON is not an object"))?;
        let components = obj
            .get("components")
            .ok_or_else(|| anyhow!("/sbom assertion: missing `components` field"))?
            .as_array()
            .ok_or_else(|| anyhow!("/sbom assertion: `components` is not an array"))?;
        if components.is_empty() {
            bail!("/sbom assertion: `components` array is empty");
        }
        Ok(())
    }
}

// ---------- compose-aware diagnostics helper ----------

/// Convenience: dump `docker compose ps` and `logs --tail 200` to stderr on
/// failure. Called by the harness, exposed here so probes that want extra
/// noise on Err can also use it. Read-only; never returns Err.
pub fn dump_diagnostics(compose_file: &PathBuf, project: &str) {
    eprintln!("[e2e] --- diagnostics: docker compose ps -----------");
    let _ = Command::new("docker")
        .args(["compose", "-f"])
        .arg(compose_file)
        .args(["-p", project, "ps"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    eprintln!("[e2e] --- diagnostics: docker compose logs --tail 200 ---");
    let mut child = match Command::new("docker")
        .args(["compose", "-f"])
        .arg(compose_file)
        .args(["-p", project, "logs", "--tail", "200"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[e2e] failed to spawn docker compose logs: {}", e);
            return;
        }
    };
    if let Some(mut out) = child.stdout.take() {
        let mut buf = String::new();
        let _ = out.read_to_string(&mut buf);
        eprintln!("{}", buf);
    }
    let _ = child.wait();
}

// ---------- per-stack probe matrix ----------

/// Return the readiness probe chain for `stack`. Probes are run in order, each
/// retried until success-or-deadline. Stacks consult `ctx.host_ports` to pick
/// the right published port; HTTP paths/credentials are baked here because
/// they are runtime-type-specific, not user-facing config.
///
/// Phase 4 environment-variable contract (chain selection):
///
/// | `FIRESTREAM_E2E_HEALTHD` | `FIRESTREAM_E2E_HTTP` | Chain                                                                            |
/// |--------------------------|-----------------------|----------------------------------------------------------------------------------|
/// | unset / `1` (default)    | unset / `1` (default) | `Tcp(<app_port>) → HealthEndpoint(/readyz + /sbom)` — preferred, single contract |
/// | unset / `1`              | `0`                   | `Tcp(<app_port>)` only (no HTTP at all; degenerate but documented)               |
/// | `0`                      | unset / `1`           | Per-protocol Phase-1 chain (PgIsReady, RedisPing, KafkaApiVersions, …)           |
/// | `0`                      | `0`                   | Per-protocol Phase-1 chain WITHOUT its HTTP component (SparkWithWorkers, …)      |
pub fn for_stack(stack: &str, ctx: &ProbeCtx) -> Vec<Box<dyn Probe_>> {
    let http_enabled = std::env::var("FIRESTREAM_E2E_HTTP")
        .map(|v| v != "0")
        .unwrap_or(true);
    let healthd_enabled = std::env::var("FIRESTREAM_E2E_HEALTHD")
        .map(|v| v != "0")
        .unwrap_or(true);

    if healthd_enabled && http_enabled {
        let app_port = canonical_app_port(stack, ctx);
        let mut chain: Vec<Box<dyn Probe_>> = Vec::with_capacity(2);
        if let Some(p) = app_port {
            chain.push(boxed(Tcp { port: p }));
        }
        if let Some(hp) = ctx.health_host_port {
            chain.push(Box::new(HealthEndpoint {
                base_url: format!("http://127.0.0.1:{}", hp),
            }));
            return chain;
        }
        eprintln!(
            "[e2e:{}] WARN: HealthEndpoint requested but health_host_port=None; \
             falling back to per-protocol chain",
            stack
        );
    }

    // Per-protocol Phase-1 chain (preserved verbatim — this is the
    // truth-table for what each runtimeType actually needs to check).
    let exec = ctx.exec();
    match stack {
        "postgresql" => {
            let pg_port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 5432)
                .unwrap_or_else(|| ctx.host_ports.first().copied().unwrap_or(5432));
            let mut chain: Vec<Box<dyn Probe_>> = vec![boxed(Tcp { port: pg_port })];
            if http_enabled {
                chain.push(boxed(PgIsReady {
                    exec: exec.clone(),
                    target: "postgresql".into(),
                    user: "postgres".into(),
                    db: "postgres".into(),
                }));
            }
            chain
        }
        "redis" => {
            let redis_port = ctx.host_ports.first().copied().unwrap_or(6379);
            vec![
                boxed(Tcp { port: redis_port }),
                boxed(RedisPing {
                    exec: exec.clone(),
                    target: "redis".into(),
                }),
            ]
        }
        "kafka" => {
            let port = ctx.host_ports.first().copied().unwrap_or(9092);
            vec![
                boxed(Tcp { port }),
                boxed(KafkaApiVersions {
                    exec: exec.clone(),
                    target: "kafka".into(),
                    bootstrap: "localhost:9092".into(),
                }),
            ]
        }
        "spark" => {
            let master_port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 8080)
                .unwrap_or_else(|| ctx.host_ports.get(1).copied().unwrap_or(8080));
            let master_url = format!("http://127.0.0.1:{}", master_port);
            let mut chain: Vec<Box<dyn Probe_>> = vec![boxed(Tcp { port: master_port })];
            if http_enabled {
                chain.push(Box::new(SparkWithWorkers {
                    master_url,
                    min_workers: 0,
                }));
            }
            chain
        }
        "airflow" => {
            let api_port: u16 = 28090;
            let mut chain: Vec<Box<dyn Probe_>> = vec![
                boxed(PgIsReady {
                    exec: exec.clone(),
                    target: "postgresql".into(),
                    user: "airflow".into(),
                    db: "airflow".into(),
                }),
                boxed(RedisPing {
                    exec: exec.clone(),
                    target: "redis".into(),
                }),
            ];
            if http_enabled {
                chain.push(boxed(HttpReady {
                    url: format!("http://127.0.0.1:{}/", api_port),
                    max_status: 500,
                }));
            }
            chain
        }
        "jupyterhub" => {
            let port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 8000)
                .unwrap_or_else(|| ctx.host_ports.first().copied().unwrap_or(8000));
            let mut chain: Vec<Box<dyn Probe_>> = vec![boxed(Tcp { port })];
            if http_enabled {
                chain.push(boxed(HttpReady {
                    url: format!("http://127.0.0.1:{}/hub/", port),
                    max_status: 500,
                }));
            }
            chain
        }
        "superset" => {
            let port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 8088)
                .unwrap_or_else(|| ctx.host_ports.first().copied().unwrap_or(8088));
            let mut chain: Vec<Box<dyn Probe_>> = vec![boxed(Tcp { port })];
            if http_enabled {
                chain.push(boxed(HttpReady {
                    url: format!("http://127.0.0.1:{}/health", port),
                    max_status: 500,
                }));
            }
            chain
        }
        "odoo" => {
            let port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 8069)
                .unwrap_or_else(|| ctx.host_ports.first().copied().unwrap_or(8069));
            let mut chain: Vec<Box<dyn Probe_>> = vec![boxed(Tcp { port })];
            if http_enabled {
                chain.push(boxed(HttpReady {
                    url: format!("http://127.0.0.1:{}/web/login", port),
                    max_status: 500,
                }));
            }
            chain
        }
        other => {
            eprintln!(
                "[e2e:{}] WARN: no probe matrix defined; falling back to TCP-only",
                other
            );
            ctx.host_ports
                .iter()
                .copied()
                .map(|p| boxed(Tcp { port: p }))
                .collect()
        }
    }
}

/// Resolve the canonical "app port" the Tcp pre-check should target for a
/// stack in the HealthEndpoint chain.
fn canonical_app_port(stack: &str, ctx: &ProbeCtx) -> Option<u16> {
    let preferred: Option<u16> = match stack {
        "postgresql" => Some(5432),
        "redis" => Some(6379),
        "kafka" => Some(9092),
        "spark" => Some(8080),
        "airflow" => Some(28090),
        "jupyterhub" => Some(8000),
        "superset" => Some(8088),
        "odoo" => Some(8069),
        _ => None,
    };
    if let Some(p) = preferred {
        if ctx.host_ports.iter().any(|x| *x == p) {
            return Some(p);
        }
    }
    ctx.host_ports.first().copied()
}

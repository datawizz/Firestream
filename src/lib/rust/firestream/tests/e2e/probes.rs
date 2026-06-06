// Per-protocol readiness probes for canonical Firestream stacks.
//
// Audit correction 3: a blanket TCP-is-bound check is near-vacuous for several
// stacks (Kafka's listener binds before metadata is serviceable; a Spark master
// binds before workers register). Each probe below tests the protocol-level
// invariant that downstream consumers actually depend on.
//
// All probes are SYNCHRONOUS. The harness drives them inside a retry loop with
// a wall-clock deadline (see `harness::retry_until_sync`). Sync probes
// eliminate the "Tokio Runtime in Drop" hazard (audit correction 6) outright:
// no async runtime is created in run_one, so there is nothing for StackGuard's
// Drop to race against.
//
// NOTE on `wait-for-port`: the upstream `PortWaiter` API is async-only.
// Spinning a fresh `tokio::runtime::Runtime` per probe invocation just to call
// `.wait().await` is wasteful and adds a Drop race. `std::net::TcpStream`
// with `connect_timeout` does the same job synchronously and is what every
// blocking probe in this file uses for connectivity checks.

use std::io::Read;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};

/// Per-probe context populated by the harness from `nix eval` data.
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
    /// Probes are advisory consumers; the harness owns retry & timeout.
    /// Allow dead_code: kept for Phase 3+ probes that bound their own
    /// per-probe operations against the outer deadline.
    #[allow(dead_code)]
    pub deadline: Instant,
}

/// Trait implemented by every readiness probe. `name` is used in failure
/// messages; `run` must be cheap and idempotent because the harness calls it
/// in a retry loop until success-or-deadline.
pub trait Probe {
    fn name(&self) -> &str;
    fn run(&self, ctx: &ProbeCtx) -> Result<()>;
}

// ---------- TCP ----------

/// Synchronous TCP "connect succeeds" probe. Tries every resolved address for
/// 127.0.0.1:port; ECONNREFUSED / ETIMEDOUT yield a retryable Err.
pub struct Tcp {
    pub port: u16,
}

impl Probe for Tcp {
    fn name(&self) -> &str {
        "tcp"
    }
    fn run(&self, _ctx: &ProbeCtx) -> Result<()> {
        let addrs: Vec<SocketAddr> = ("127.0.0.1", self.port)
            .to_socket_addrs()
            .with_context(|| format!("resolve 127.0.0.1:{}", self.port))?
            .collect();
        let timeout = Duration::from_secs(2);
        let mut last_err: Option<std::io::Error> = None;
        for addr in &addrs {
            match TcpStream::connect_timeout(addr, timeout) {
                Ok(_) => return Ok(()),
                Err(e) => last_err = Some(e),
            }
        }
        Err(anyhow!(
            "tcp connect 127.0.0.1:{} failed: {}",
            self.port,
            last_err
                .map(|e| e.to_string())
                .unwrap_or_else(|| "no addresses".into())
        ))
    }
}

// ---------- pg_isready ----------

/// `docker compose exec -T <service> pg_isready -U <user> -d <db>` until exit 0.
pub struct PgIsReady {
    pub service: String,
    pub user: String,
    pub db: String,
}

impl Probe for PgIsReady {
    fn name(&self) -> &str {
        "pg_isready"
    }
    fn run(&self, ctx: &ProbeCtx) -> Result<()> {
        let status = Command::new("docker")
            .args(["compose", "-f"])
            .arg(&ctx.compose_file)
            .args(["-p", &ctx.project, "exec", "-T", &self.service, "pg_isready", "-U", &self.user, "-d", &self.db])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("spawn docker compose exec pg_isready")?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("pg_isready exited with {}", status))
        }
    }
}

// ---------- redis ping ----------

/// `docker compose exec -T <service> redis-cli ping` until output == "PONG".
pub struct RedisPing {
    pub service: String,
}

impl Probe for RedisPing {
    fn name(&self) -> &str {
        "redis_ping"
    }
    fn run(&self, ctx: &ProbeCtx) -> Result<()> {
        let out = Command::new("docker")
            .args(["compose", "-f"])
            .arg(&ctx.compose_file)
            .args(["-p", &ctx.project, "exec", "-T", &self.service, "redis-cli", "ping"])
            .stdin(Stdio::null())
            .output()
            .context("spawn docker compose exec redis-cli ping")?;
        if !out.status.success() {
            bail!("redis-cli ping exit {}: {}", out.status, String::from_utf8_lossy(&out.stderr));
        }
        let body = String::from_utf8_lossy(&out.stdout);
        if body.trim() == "PONG" {
            Ok(())
        } else {
            Err(anyhow!("redis-cli ping returned {:?}", body))
        }
    }
}

// ---------- kafka broker metadata ----------

/// `kafka-broker-api-versions --bootstrap-server <bootstrap>` until exit 0.
/// Verifies broker metadata is serviceable, not just listener bound.
pub struct KafkaApiVersions {
    pub service: String,
    pub bootstrap: String,
}

impl Probe for KafkaApiVersions {
    fn name(&self) -> &str {
        "kafka_api_versions"
    }
    fn run(&self, ctx: &ProbeCtx) -> Result<()> {
        let status = Command::new("docker")
            .args(["compose", "-f"])
            .arg(&ctx.compose_file)
            .args([
                "-p",
                &ctx.project,
                "exec",
                "-T",
                &self.service,
                "kafka-broker-api-versions",
                "--bootstrap-server",
                &self.bootstrap,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("spawn docker compose exec kafka-broker-api-versions")?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("kafka-broker-api-versions exited with {}", status))
        }
    }
}

// ---------- spark master + worker count ----------

/// HTTP GET on master REST endpoint, parse JSON, require `aliveworkers >= min_workers`.
/// Reference: Spark master serves `/json/` returning `{ aliveworkers, ... }`.
pub struct SparkWithWorkers {
    pub master_url: String, // e.g. "http://127.0.0.1:8080"
    pub min_workers: u32,
}

impl Probe for SparkWithWorkers {
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

// ---------- generic HTTP readiness ----------

/// `reqwest::blocking::Client` GET; redirect policy = `none`; ready iff
/// connect succeeds AND status < max_status (so 200/301/302/401/403 all count
/// as "the app is alive"; 5xx is not ready).
pub struct HttpReady {
    pub url: String,
    pub max_status: u16,
}

impl Probe for HttpReady {
    fn name(&self) -> &str {
        "http_ready"
    }
    fn run(&self, _ctx: &ProbeCtx) -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .context("build reqwest blocking client")?;
        // reqwest converts connect failures (ECONNREFUSED, ETIMEDOUT) into Err,
        // which the outer retry loop will swallow until the deadline.
        let resp = client
            .get(&self.url)
            .send()
            .with_context(|| format!("GET {}", self.url))?;
        let status = resp.status().as_u16();
        if status < self.max_status {
            Ok(())
        } else {
            Err(anyhow!("GET {} status {} >= max {}", self.url, status, self.max_status))
        }
    }
}

// ---------- firestream-healthd /readyz + /sbom ----------

/// Probe the in-image `firestream-healthd` service. Polls `<base_url>/readyz`
/// until 200, then makes a single `<base_url>/sbom?format=cyclonedx` request
/// and asserts the document is a JSON object containing a non-empty
/// `components` array. The /sbom check happens once on the first /readyz
/// success — the harness retry loop is only responsible for the readiness
/// portion; a /sbom failure escalates immediately because the SBOM is baked
/// at build time and retries cannot fix a malformed document.
///
/// Phase 3: only postgres is wired to this probe. Other stacks remain on
/// their Phase 1 per-protocol probes until Phase 4.
pub struct HealthEndpoint {
    pub base_url: String,
}

impl Probe for HealthEndpoint {
    fn name(&self) -> &str {
        "health-endpoint"
    }
    fn run(&self, _ctx: &ProbeCtx) -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .context("build reqwest blocking client")?;

        // Step 1: /readyz must return 200. reqwest converts connect failures
        // (ECONNREFUSED, ETIMEDOUT) into Err which the outer retry loop in
        // harness::retry_until_sync swallows until the deadline.
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
        // non-empty `components` array. CycloneDX 1.5 puts components at the
        // top level (`{"bomFormat":"CycloneDX","components":[…]}`).
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
///
/// `HEALTHD=0` is the principal escape hatch: it forces the harness to use
/// the per-protocol probes (the Phase-1 truth-table for what each readiness
/// command actually checks). `HTTP=0` then further strips any HTTP-based
/// probe out of whichever chain `HEALTHD` selected.
pub fn for_stack(stack: &str, ctx: &ProbeCtx) -> Vec<Box<dyn Probe>> {
    let http_enabled = std::env::var("FIRESTREAM_E2E_HTTP")
        .map(|v| v != "0")
        .unwrap_or(true);
    // HEALTHD=0 disables the unified HealthEndpoint chain everywhere and
    // falls back to per-protocol probes (see the truth-table in the doc
    // comment above). Default = use HealthEndpoint.
    let healthd_enabled = std::env::var("FIRESTREAM_E2E_HEALTHD")
        .map(|v| v != "0")
        .unwrap_or(true);

    // Phase 4: when HEALTHD is enabled (the default) AND HTTP is enabled,
    // every canonical stack uses the same chain shape:
    //     Tcp(<app_port>) → HealthEndpoint(http://127.0.0.1:<health_host_port>)
    // The HealthEndpoint probe asserts both /readyz=200 AND
    // /sbom?format=cyclonedx parses to a JSON object with a non-empty
    // `components` array (see HealthEndpoint::run). This is the "one
    // contract everywhere" form.
    //
    // When HEALTHD=0, fall through to the Phase-1 per-protocol matrix
    // (retained verbatim below) so probes.rs remains the truth-table.
    if healthd_enabled && http_enabled {
        let app_port = canonical_app_port(stack, ctx);
        let mut chain: Vec<Box<dyn Probe>> = Vec::with_capacity(2);
        if let Some(p) = app_port {
            chain.push(Box::new(Tcp { port: p }));
        }
        if let Some(hp) = ctx.health_host_port {
            chain.push(Box::new(HealthEndpoint {
                base_url: format!("http://127.0.0.1:{}", hp),
            }));
            return chain;
        }
        // health_host_port is None means the stack has health.enable=false
        // at the Nix layer. In Phase 4 every canonical stack opts in, so
        // this branch is essentially defensive; fall through to per-protocol.
        eprintln!(
            "[e2e:{}] WARN: HealthEndpoint requested but health_host_port=None; \
             falling back to per-protocol chain",
            stack
        );
    }

    // Per-protocol Phase-1 chain (preserved verbatim — this is the
    // truth-table for what each runtimeType actually needs to check).
    // Reached when:
    //   - FIRESTREAM_E2E_HEALTHD=0 (explicit opt-out of HealthEndpoint), or
    //   - FIRESTREAM_E2E_HTTP=0 (degenerate TCP-only — but if HEALTHD is
    //     also on, the chain above already produced just the Tcp probe and
    //     returned), or
    //   - health_host_port=None at runtime (defensive fall-through).
    match stack {
        "postgresql" => {
            let pg_port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 5432)
                .unwrap_or_else(|| ctx.host_ports.first().copied().unwrap_or(5432));
            let mut chain: Vec<Box<dyn Probe>> = vec![Box::new(Tcp { port: pg_port })];
            if http_enabled {
                chain.push(Box::new(PgIsReady {
                    service: "postgresql".into(),
                    user: "postgres".into(),
                    db: "postgres".into(),
                }));
            }
            chain
        }
        "redis" => {
            let redis_port = ctx.host_ports.first().copied().unwrap_or(6379);
            vec![
                Box::new(Tcp { port: redis_port }),
                Box::new(RedisPing {
                    service: "redis".into(),
                }),
            ]
        }
        "kafka" => {
            // Probe broker metadata via the in-container kafka tool. The
            // listener may bind well before metadata is serviceable, so the
            // bootstrap port alone is not enough.
            let port = ctx.host_ports.first().copied().unwrap_or(9092);
            vec![
                Box::new(Tcp { port }),
                Box::new(KafkaApiVersions {
                    service: "kafka".into(),
                    bootstrap: "localhost:9092".into(),
                }),
            ]
        }
        "spark" => {
            // exposedPorts: [7077, 8080, 8081, 4040, 6066]. The master UI is on
            // host 8080 (single-service default branch, offset 0). The worker
            // UI is on 8081 — but in the default single-service compose there
            // is no worker. For Phase 1 we only assert the master answers and
            // require zero workers; later phases will introduce a worker.
            //
            // NOTE: the default compose has ONE service called "spark" which
            // runs the master. There is no separate worker yet, so
            // `min_workers = 0` here is intentional.
            let master_port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 8080)
                .unwrap_or_else(|| ctx.host_ports.get(1).copied().unwrap_or(8080));
            let master_url = format!("http://127.0.0.1:{}", master_port);
            let mut chain: Vec<Box<dyn Probe>> = vec![Box::new(Tcp { port: master_port })];
            if http_enabled {
                chain.push(Box::new(SparkWithWorkers {
                    master_url,
                    min_workers: 0,
                }));
            }
            chain
        }
        "airflow" => {
            // Airflow stack publishes 25432 (pg), 26379 (redis), 28090 (api-server),
            // 29180 (healthd). The hostPorts list is [api-server, healthd, pg,
            // redis] in our setup (offset 20000). We probe pg/redis via docker
            // exec, plus an HTTP probe on the api-server (8090 + 20000 = 28090).
            let api_port: u16 = 28090;
            let mut chain: Vec<Box<dyn Probe>> = vec![
                Box::new(PgIsReady {
                    service: "postgresql".into(),
                    user: "airflow".into(),
                    db: "airflow".into(),
                }),
                Box::new(RedisPing {
                    service: "redis".into(),
                }),
            ];
            if http_enabled {
                chain.push(Box::new(HttpReady {
                    url: format!("http://127.0.0.1:{}/", api_port),
                    max_status: 500,
                }));
            }
            chain
        }
        "jupyterhub" => {
            // exposedPorts = [8000 8081]; hub UI on 8000.
            let port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 8000)
                .unwrap_or_else(|| ctx.host_ports.first().copied().unwrap_or(8000));
            let mut chain: Vec<Box<dyn Probe>> = vec![Box::new(Tcp { port })];
            if http_enabled {
                chain.push(Box::new(HttpReady {
                    url: format!("http://127.0.0.1:{}/hub/", port),
                    max_status: 500,
                }));
            }
            chain
        }
        "superset" => {
            // exposedPorts = [8088 5555]; webserver on 8088.
            let port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 8088)
                .unwrap_or_else(|| ctx.host_ports.first().copied().unwrap_or(8088));
            let mut chain: Vec<Box<dyn Probe>> = vec![Box::new(Tcp { port })];
            if http_enabled {
                chain.push(Box::new(HttpReady {
                    url: format!("http://127.0.0.1:{}/health", port),
                    max_status: 500,
                }));
            }
            chain
        }
        "odoo" => {
            // exposedPorts = [8069 8072]; web UI on 8069.
            let port = ctx
                .host_ports
                .iter()
                .copied()
                .find(|p| *p == 8069)
                .unwrap_or_else(|| ctx.host_ports.first().copied().unwrap_or(8069));
            let mut chain: Vec<Box<dyn Probe>> = vec![Box::new(Tcp { port })];
            if http_enabled {
                chain.push(Box::new(HttpReady {
                    url: format!("http://127.0.0.1:{}/web/login", port),
                    max_status: 500,
                }));
            }
            chain
        }
        other => {
            // Defensive: undeclared stack falls back to TCP-only on every
            // published port. Should never trigger because `stacks::CANONICAL`
            // and the macro in `tests/e2e.rs` stay in sync; if it does, the
            // probe is at least non-vacuous (it will catch nothing binding).
            eprintln!(
                "[e2e:{}] WARN: no probe matrix defined; falling back to TCP-only",
                other
            );
            ctx.host_ports
                .iter()
                .copied()
                .map(|p| Box::new(Tcp { port: p }) as Box<dyn Probe>)
                .collect()
        }
    }
}

/// Resolve the canonical "app port" the Tcp pre-check should target for a
/// stack in the HealthEndpoint chain. Mirrors the port-pick logic in the
/// per-protocol matrix so the Tcp pre-check fails fast when the user-facing
/// listener isn't bound yet (independent of /readyz).
fn canonical_app_port(stack: &str, ctx: &ProbeCtx) -> Option<u16> {
    // Tries an exact match against the stack's well-known port first;
    // falls back to the first published port; returns None only when the
    // stack publishes nothing (shouldn't happen for any canonical stack).
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

//! firestream-healthd: tiny in-image HTTP server that exposes the baked
//! container metadata + a configurable readiness probe.
//!
//! Design constraints:
//! - No tokio, no axum, no hyper, no TLS. Plain blocking HTTP via `tiny_http`.
//! - One std::thread per request (tiny_http is blocking).
//! - Readiness is configured *once* at process start; the binary itself stays
//!   generic across container runtime types.
//! - Reads pre-baked files from `--metadata-path` (default `/opt/firestream`)
//!   without ever regenerating SBOMs at runtime.

mod readiness;
mod routes;

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::readiness::ReadinessCheck;
use crate::routes::Ctx;

#[derive(Debug, Parser)]
#[command(
    name = "firestream-healthd",
    version,
    about = "In-container health/SBOM HTTP server for Firestream"
)]
struct Cli {
    /// TCP port to listen on.
    #[arg(long, env = "FIRESTREAM_HEALTHD_PORT", default_value_t = 9180)]
    port: u16,

    /// Address to bind. Defaults to all interfaces so callers in the same
    /// container, sibling containers in a compose network, or the host can
    /// reach the server when the port is published.
    #[arg(long, env = "FIRESTREAM_HEALTHD_BIND", default_value = "0.0.0.0")]
    bind: String,

    /// Base directory where the baked metadata files (`metadata.json`,
    /// `sbom-*.json`, `closure.json`) live. Containers receive these at
    /// `/opt/firestream` from the Nix build.
    #[arg(
        long,
        env = "FIRESTREAM_HEALTHD_METADATA_PATH",
        default_value = "/opt/firestream"
    )]
    metadata_path: PathBuf,

    /// Run `sh -c <cmd>` for readiness. Ready iff the command exits 0.
    #[arg(long, env = "FIRESTREAM_HEALTHD_READINESS_CMD")]
    readiness_cmd: Option<String>,

    /// Open a TCP connection to `localhost:<n>` for readiness.
    #[arg(long, env = "FIRESTREAM_HEALTHD_READINESS_PORT")]
    readiness_port: Option<u16>,

    /// HTTP GET <url> for readiness. Ready iff connect succeeds and status < 500.
    #[arg(long, env = "FIRESTREAM_HEALTHD_READINESS_HTTP")]
    readiness_http: Option<String>,
}

fn main() -> anyhow::Result<()> {
    // Logs go to stderr (stdout is reserved for whatever app the container
    // entrypoint might also be writing). Default filter is INFO; override via
    // RUST_LOG.
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr))
        .init();

    let cli = Cli::parse();

    // Build the readiness check from the (mutually-exclusive) flags. If more
    // than one is set we take the first non-None in declaration order and log
    // a warning, rather than erroring — operational ergonomics over strictness.
    let readiness = match (
        cli.readiness_cmd.as_deref(),
        cli.readiness_port,
        cli.readiness_http.as_deref(),
    ) {
        (Some(c), p, h) => {
            if p.is_some() || h.is_some() {
                tracing::warn!("multiple readiness flags set; using --readiness-cmd");
            }
            ReadinessCheck::Cmd(c.to_string())
        }
        (None, Some(p), h) => {
            if h.is_some() {
                tracing::warn!("multiple readiness flags set; using --readiness-port");
            }
            ReadinessCheck::Tcp(p)
        }
        (None, None, Some(u)) => ReadinessCheck::Http(u.to_string()),
        (None, None, None) => ReadinessCheck::None,
    };

    tracing::info!(
        port = cli.port,
        bind = %cli.bind,
        metadata_path = %cli.metadata_path.display(),
        readiness = readiness.name(),
        "starting firestream-healthd"
    );

    let addr = format!("{}:{}", cli.bind, cli.port);
    let server = tiny_http::Server::http(&addr)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
        .with_context(|| format!("bind {}", addr))?;

    let ctx = Arc::new(Ctx {
        metadata_path: cli.metadata_path,
        readiness,
    });

    for request in server.incoming_requests() {
        let ctx = Arc::clone(&ctx);
        // One thread per request — tiny_http is blocking, but for the volume
        // of traffic a health endpoint receives this is fine.
        thread::spawn(move || routes::handle(request, &ctx));
    }

    Ok(())
}

use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

use wait_for_port::{PortState, PortWaiter, WaitForPortError};

/// Wait for a TCP port to reach a desired state
///
/// This tool polls a TCP port until it reaches the desired state (inuse or free),
/// with a configurable timeout. Useful for container orchestration and service
/// startup sequencing.
///
/// Exit codes:
///   0 - Port reached the desired state
///   1 - Timeout or error
#[derive(Parser, Debug)]
#[command(name = "wait-for-port")]
#[command(version)]
#[command(about = "Wait for a TCP port to reach a desired state")]
struct Cli {
    /// Port number to check (1-65535)
    port: u16,

    /// Target host (IP address or hostname)
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// Desired port state: 'inuse' (service listening) or 'free' (port available)
    #[arg(short, long, default_value = "inuse", value_parser = parse_state)]
    state: PortState,

    /// Timeout in seconds (0 = no timeout)
    #[arg(short, long, default_value = "30")]
    timeout: u64,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn parse_state(s: &str) -> Result<PortState, String> {
    PortState::from_str(s).ok_or_else(|| {
        format!(
            "Invalid state '{}'. Valid values are: inuse, free",
            s
        )
    })
}

fn init_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("wait_for_port=debug")
    } else {
        EnvFilter::new("wait_for_port=info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .init();
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    init_logging(cli.verbose);

    // Validate port
    if cli.port == 0 {
        error!("Invalid port number: 0 (must be 1-65535)");
        return ExitCode::from(1);
    }

    let timeout = if cli.timeout == 0 {
        // Very long timeout (effectively no timeout)
        Duration::from_secs(u64::MAX / 2)
    } else {
        Duration::from_secs(cli.timeout)
    };

    debug!(
        "Waiting for port {} on {} to be {} (timeout: {}s)",
        cli.port, cli.host, cli.state, cli.timeout
    );

    let result = PortWaiter::new(cli.port)
        .host(&cli.host)
        .state(cli.state)
        .timeout(timeout)
        .wait()
        .await;

    match result {
        Ok(()) => {
            info!(
                "Port {} on {} is {}",
                cli.port, cli.host, cli.state
            );
            ExitCode::SUCCESS
        }
        Err(WaitForPortError::Timeout { host, port, state, timeout_secs }) => {
            error!(
                "Timeout after {}s waiting for port {} on {} to be {}",
                timeout_secs, port, host, state
            );
            ExitCode::from(1)
        }
        Err(WaitForPortError::InvalidPort(port)) => {
            error!("Invalid port number: {} (must be 1-65535)", port);
            ExitCode::from(1)
        }
        Err(WaitForPortError::DnsResolutionFailed(host)) => {
            error!("Cannot resolve host: {}", host);
            ExitCode::from(1)
        }
        Err(WaitForPortError::Io(e)) => {
            error!("IO error: {}", e);
            ExitCode::from(1)
        }
    }
}

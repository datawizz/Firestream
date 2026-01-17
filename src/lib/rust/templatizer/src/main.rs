//! Templatizer CLI entry point
//!
//! This is the main entry point for the templatizer command-line tool.
//! It parses arguments and delegates to the appropriate generator.

use templatizer::cli::{self, Cli};
use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    // Parse CLI arguments
    let cli = Cli::parse_args();

    // Run the command
    if let Err(e) = cli::run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

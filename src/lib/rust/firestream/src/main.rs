use clap::Parser;
use firestream::{
    cli::{Cli, execute_command},
    config::init_config_dir,
    core::error_to_exit_code,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));
    
    if !cli.quiet {
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(filter)
            .init();
    }
    
    // Initialize configuration directory
    if let Err(e) = init_config_dir().await {
        eprintln!("Failed to initialize configuration directory: {}", e);
        std::process::exit(1);
    }
    
    // Execute command
    match execute_command(cli).await {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(error_to_exit_code(&e));
        }
    }
}

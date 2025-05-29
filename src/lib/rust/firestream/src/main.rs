use clap::Parser;
use firestream::{
    cli::{Cli, Command, execute_command},
    config::init_config_dir,
    core::error_to_exit_code,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::sync::{Arc, Mutex};

// Custom writer for tracing to handle file logging in TUI mode
struct LogWriter {
    file: Arc<Mutex<File>>,
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut file = self.file.lock().unwrap();
        file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut file = self.file.lock().unwrap();
        file.flush()
    }
}

impl Clone for LogWriter {
    fn clone(&self) -> Self {
        LogWriter {
            file: self.file.clone(),
        }
    }
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Check if we're running TUI mode
    let is_tui = matches!(cli.command, None | Some(Command::Tui));
    
    // Initialize logging
    if !is_tui {
        // Regular CLI mode - log to stdout
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
    } else {
        // TUI mode - redirect logs to a file to avoid terminal corruption
        // Try to create .firestream directory first
        let _ = create_dir_all(".firestream");
        
        let log_path = if let Ok(file) = File::create(".firestream/tui.log") {
            Arc::new(Mutex::new(file))
        } else {
            Arc::new(Mutex::new(File::create("/tmp/firestream-tui.log")
                .expect("Failed to create log file")))
        };
        
        let file_writer = log_path.clone();
        let file_layer = fmt::layer()
            .with_writer(move || {
                let file = file_writer.clone();
                LogWriter { file }
            })
            .with_ansi(false);
        
        tracing_subscriber::registry()
            .with(file_layer)
            .with(EnvFilter::new("info"))
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

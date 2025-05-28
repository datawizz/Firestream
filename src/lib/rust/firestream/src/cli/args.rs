//! Command-line argument definitions
//!
//! This module defines the CLI structure using clap.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Firestream CLI - Data Infrastructure Management Tool
#[derive(Parser, Debug)]
#[command(name = "firestream")]
#[command(about = "CLI/TUI tool for managing data infrastructure services", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Path to global config file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Kubernetes namespace
    #[arg(short, long, global = true)]
    pub namespace: Option<String>,

    /// Increase verbosity
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install a service
    Install {
        /// Service name
        service: String,
        /// Custom configuration file
        #[arg(long)]
        config: Option<PathBuf>,
    },
    
    /// Uninstall a service
    Uninstall {
        /// Service name
        service: String,
    },
    
    /// Start a service
    Start {
        /// Service name
        service: String,
    },
    
    /// Stop a service
    Stop {
        /// Service name
        service: String,
    },
    
    /// Restart a service
    Restart {
        /// Service name
        service: String,
    },
    
    /// Show status of services
    Status {
        /// Service name (shows all if not specified)
        service: Option<String>,
    },
    
    /// List services
    List {
        /// Show available services instead of installed
        #[arg(long)]
        available: bool,
    },
    
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    
    /// View service logs
    Logs {
        /// Service name
        service: String,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        lines: usize,
    },
    
    /// Show resource usage
    Resources {
        /// Service name (shows all if not specified)
        service: Option<String>,
    },
    
    /// Launch the TUI interface
    #[command(name = "tui")]
    Tui,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Show configuration
    Show {
        /// Service name
        service: String,
    },
    
    /// Edit configuration
    Edit {
        /// Service name
        service: String,
    },
    
    /// Validate configuration
    Validate {
        /// Service name or config file path
        target: String,
    },
}

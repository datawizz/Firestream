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
    /// Initialize a new Firestream project
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
    },
    
    /// Generate execution plan
    Plan {
        /// Target specific resources
        #[arg(long)]
        target: Vec<String>,
        
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        output: String,
        
        /// Save plan to file
        #[arg(long)]
        out: Option<PathBuf>,
    },
    
    /// Apply changes from plan
    Apply {
        /// Plan ID or file
        #[arg(long)]
        plan: Option<String>,
        
        /// Auto-approve changes
        #[arg(long)]
        auto_approve: bool,
        
        /// Target specific resources
        #[arg(long)]
        target: Vec<String>,
    },
    
    /// Refresh state from actual resources
    Refresh,
    
    /// Import existing resources
    Import {
        /// Resource type (infrastructure, build, deployment, cluster)
        resource_type: String,
        
        /// Resource ID
        resource_id: String,
        
        /// Resource data file (JSON)
        #[arg(long)]
        data: Option<PathBuf>,
    },
    
    /// Manage state
    State {
        #[command(subcommand)]
        command: StateCommand,
    },
    
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
    
    /// Manage Kubernetes clusters
    Cluster {
        #[command(subcommand)]
        command: ClusterCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum ClusterCommand {
    /// Create a new k3d cluster
    Create {
        /// Cluster name
        #[arg(long)]
        name: Option<String>,
        
        /// Use configuration from file
        #[arg(short, long)]
        config: Option<PathBuf>,
        
        /// Clean mode - delete existing cluster first
        #[arg(long)]
        clean: bool,
        
        /// Number of server nodes
        #[arg(long, default_value = "1")]
        servers: u32,
        
        /// Number of agent nodes
        #[arg(long, default_value = "1")]
        agents: u32,
        
        /// Enable development mode with port forwarding
        #[arg(long)]
        dev_mode: bool,
    },
    
    /// Delete k3d cluster
    Delete {
        /// Cluster name
        name: Option<String>,
    },
    
    /// Show cluster info
    Info {
        /// Cluster name
        name: Option<String>,
    },
    
    /// Setup port forwarding for development
    PortForward {
        /// Service name (or "all" for all services)
        #[arg(default_value = "all")]
        service: String,
        
        /// Port offset for external ports
        #[arg(long, default_value = "10000")]
        offset: u16,
    },
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

#[derive(Subcommand, Debug)]
pub enum StateCommand {
    /// Show current state
    Show {
        /// Show specific resource
        #[arg(long)]
        resource: Option<String>,
        
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        output: String,
    },
    
    /// Lock state
    Lock {
        /// Lock timeout in seconds
        #[arg(long, default_value = "300")]
        timeout: u64,
    },
    
    /// Unlock state
    Unlock {
        /// Force unlock
        #[arg(long)]
        force: bool,
    },
    
    /// List state locks
    Locks,
    
    /// Pull remote state
    Pull,
    
    /// Push local state to remote
    Push,
}

//! K8s Manager CLI
//!
//! Command-line interface for managing Kubernetes clusters across multiple providers.

use clap::{Parser, Subcommand};
use k8s_manager::{
    K3dClusterManager, K3dClusterConfig, K3dConfig, K3dDevModeConfig,
    ClusterManager, ClusterLifecycle, ClusterNetworking, ClusterObservability,
    ClusterSecurity,
    PortForwardConfig, LogsConfig, ResourceType, DiagnosticsConfig,
};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "k8s-manager")]
#[command(about = "Kubernetes cluster management across multiple providers", long_about = None)]
#[command(version)]
struct Cli {
    /// Set the verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Provider to use (currently only k3d is implemented)
    #[arg(short, long, default_value = "k3d")]
    provider: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage clusters
    Cluster {
        #[command(subcommand)]
        action: ClusterCommands,
    },
    /// Manage registries
    Registry {
        #[command(subcommand)]
        action: RegistryCommands,
    },
    /// Port forwarding operations
    PortForward {
        /// Namespace
        #[arg(short, long, default_value = "default")]
        namespace: String,
        /// Service name
        service: String,
        /// Local port
        #[arg(short, long)]
        local_port: u16,
        /// Remote port
        #[arg(short, long)]
        remote_port: u16,
    },
    /// View logs from resources
    Logs {
        /// Resource type (pod, deployment, service, etc.)
        #[arg(short = 't', long, default_value = "pod")]
        resource_type: String,
        /// Resource name
        resource: String,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
        /// Container name
        #[arg(short, long)]
        container: Option<String>,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        /// Show previous container logs
        #[arg(short, long)]
        previous: bool,
        /// Show logs from all containers
        #[arg(short, long)]
        all_containers: bool,
    },
    /// Get cluster diagnostics
    Diagnostics {
        /// Namespace (empty for all namespaces)
        #[arg(short, long)]
        namespace: Option<String>,
        /// Include nodes
        #[arg(long)]
        nodes: bool,
        /// Include pods
        #[arg(long)]
        pods: bool,
        /// Include services
        #[arg(long)]
        services: bool,
        /// Include events
        #[arg(long)]
        events: bool,
        /// Include all resources
        #[arg(short, long)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum ClusterCommands {
    /// Create a new cluster
    Create {
        /// Cluster name
        #[arg(default_value = "firestream")]
        name: String,
        /// API port
        #[arg(long, default_value = "6550")]
        api_port: u16,
        /// HTTP port
        #[arg(long, default_value = "80")]
        http_port: u16,
        /// HTTPS port
        #[arg(long, default_value = "443")]
        https_port: u16,
        /// Number of server nodes
        #[arg(long, default_value = "1")]
        servers: u32,
        /// Number of agent nodes
        #[arg(long, default_value = "1")]
        agents: u32,
        /// Enable development mode
        #[arg(long)]
        dev_mode: bool,
        /// Port offset for dev mode
        #[arg(long, default_value = "20000")]
        port_offset: u16,
        /// Disable TLS setup
        #[arg(long)]
        no_tls: bool,
        /// Disable network configuration
        #[arg(long)]
        no_network: bool,
        /// Use basic setup (minimal configuration)
        #[arg(long)]
        basic: bool,
    },
    /// Delete a cluster
    Delete {
        /// Cluster name
        name: String,
    },
    /// List all clusters
    List,
    /// Get cluster information
    Info {
        /// Cluster name
        name: String,
    },
    /// Start a stopped cluster
    Start {
        /// Cluster name
        name: String,
    },
    /// Stop a running cluster
    Stop {
        /// Cluster name
        name: String,
    },
    /// Restart a cluster
    Restart {
        /// Cluster name
        name: String,
    },
    /// Setup a cluster with full features (create if needed, configure all features)
    Setup {
        /// Cluster name
        #[arg(default_value = "firestream")]
        name: String,
        /// Use default configuration
        #[arg(long)]
        defaults: bool,
    },
}

#[derive(Subcommand)]
enum RegistryCommands {
    /// Create a registry
    Create {
        /// Registry name
        #[arg(default_value = "registry.localhost")]
        name: String,
        /// Registry port
        #[arg(short, long, default_value = "5000")]
        port: u16,
    },
    /// Delete a registry
    Delete {
        /// Registry name
        name: String,
    },
    /// List registries
    List,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Currently only k3d is implemented
    if cli.provider != "k3d" {
        error!("Provider '{}' is not yet implemented. Currently only 'k3d' is available.", cli.provider);
        return Ok(());
    }

    match cli.command {
        Commands::Cluster { action } => handle_cluster_command(action).await?,
        Commands::Registry { action } => handle_registry_command(action).await?,
        Commands::PortForward {
            namespace,
            service,
            local_port,
            remote_port,
        } => {
            handle_port_forward(namespace, service, local_port, remote_port).await?;
        }
        Commands::Logs {
            resource_type,
            resource,
            namespace,
            container,
            follow,
            previous,
            all_containers,
        } => {
            handle_logs(
                resource_type,
                resource,
                namespace,
                container,
                follow,
                previous,
                all_containers,
            )
            .await?;
        }
        Commands::Diagnostics {
            namespace,
            nodes,
            pods,
            services,
            events,
            all,
        } => {
            handle_diagnostics(namespace, nodes, pods, services, events, all).await?;
        }
    }

    Ok(())
}

async fn handle_cluster_command(command: ClusterCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ClusterCommands::Create {
            name,
            api_port,
            http_port,
            https_port,
            servers,
            agents,
            dev_mode,
            port_offset,
            no_tls,
            no_network,
            basic,
        } => {
            if basic {
                // Use basic setup
                let config = K3dConfig {
                    cluster_name: name,
                    api_port,
                    lb_port: http_port,
                    agents,
                    servers,
                    ..Default::default()
                };
                info!("Creating cluster with basic configuration...");
                k8s_manager::k3d::setup_cluster_with_config(&config).await?;
            } else {
                // Use advanced setup
                let mut config = K3dClusterConfig {
                    name: name.clone(),
                    api_port,
                    http_port,
                    https_port,
                    servers,
                    agents,
                    ..Default::default()
                };

                if dev_mode {
                    config.dev_mode = Some(K3dDevModeConfig {
                        port_forward_all: true,
                        port_offset,
                    });
                }

                config.tls.enabled = !no_tls;
                config.network.configure_routes = !no_network;
                config.network.configure_dns = !no_network;
                config.network.patch_etc_hosts = !no_network;

                let manager = K3dClusterManager::new(config);
                info!("Creating cluster '{}'...", name);
                manager.create_cluster().await?;

                if !no_tls || !no_network {
                    info!("Configuring additional features...");
                    if !no_tls {
                        manager.configure_tls().await?;
                    }
                    if !no_network {
                        manager.setup_routes().await?;
                        manager.configure_dns().await?;
                    }
                }
            }
            info!("Cluster created successfully!");
        }
        ClusterCommands::Delete { name } => {
            info!("Deleting cluster '{}'...", name);
            k8s_manager::k3d::delete_cluster(&name).await?;
            info!("Cluster deleted successfully!");
        }
        ClusterCommands::List => {
            info!("Listing clusters...");
            // Use k3d CLI directly for now
            let output = tokio::process::Command::new("k3d")
                .args(&["cluster", "list"])
                .output()
                .await?;
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        ClusterCommands::Info { name } => {
            let config = K3dClusterConfig {
                name: name.clone(),
                ..Default::default()
            };
            let manager = K3dClusterManager::new(config);
            let info = manager.get_cluster_info().await?;
            
            println!("Cluster Information:");
            println!("  Name: {}", info.name);
            println!("  Provider: {:?}", info.provider);
            println!("  Status: {:?}", info.status);
            if let Some(endpoint) = info.endpoint {
                println!("  Endpoint: {}", endpoint);
            }
            if let Some(version) = info.kubernetes_version {
                println!("  Kubernetes Version: {}", version);
            }
            println!("  Node Count: {}", info.node_count);
            if !info.metadata.is_empty() {
                println!("  Metadata:");
                for (key, value) in info.metadata {
                    println!("    {}: {}", key, value);
                }
            }
        }
        ClusterCommands::Start { name } => {
            let config = K3dClusterConfig {
                name: name.clone(),
                ..Default::default()
            };
            let manager = K3dClusterManager::new(config);
            info!("Starting cluster '{}'...", name);
            manager.start_cluster().await?;
            info!("Cluster started successfully!");
        }
        ClusterCommands::Stop { name } => {
            let config = K3dClusterConfig {
                name: name.clone(),
                ..Default::default()
            };
            let manager = K3dClusterManager::new(config);
            info!("Stopping cluster '{}'...", name);
            manager.stop_cluster().await?;
            info!("Cluster stopped successfully!");
        }
        ClusterCommands::Restart { name } => {
            let config = K3dClusterConfig {
                name: name.clone(),
                ..Default::default()
            };
            let manager = K3dClusterManager::new(config);
            info!("Restarting cluster '{}'...", name);
            manager.restart_cluster().await?;
            info!("Cluster restarted successfully!");
        }
        ClusterCommands::Setup { name, defaults } => {
            if defaults {
                info!("Setting up cluster with default configuration...");
                k8s_manager::k3d::setup_cluster().await?;
            } else {
                let config = K3dClusterConfig {
                    name: name.clone(),
                    ..Default::default()
                };
                let manager = K3dClusterManager::new(config);
                info!("Setting up cluster '{}' with full features...", name);
                manager.setup_cluster().await?;
            }
            info!("Cluster setup complete!");
        }
    }
    Ok(())
}

async fn handle_registry_command(command: RegistryCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        RegistryCommands::Create { name, port } => {
            info!("Creating registry '{}' on port {}...", name, port);
            let output = tokio::process::Command::new("k3d")
                .args(&[
                    "registry",
                    "create",
                    &name,
                    "--port",
                    &port.to_string(),
                ])
                .output()
                .await?;
            
            if output.status.success() {
                info!("Registry created successfully!");
            } else {
                error!("Failed to create registry: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        RegistryCommands::Delete { name } => {
            info!("Deleting registry '{}'...", name);
            let output = tokio::process::Command::new("k3d")
                .args(&["registry", "delete", &name])
                .output()
                .await?;
            
            if output.status.success() {
                info!("Registry deleted successfully!");
            } else {
                error!("Failed to delete registry: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        RegistryCommands::List => {
            info!("Listing registries...");
            let output = tokio::process::Command::new("k3d")
                .args(&["registry", "list"])
                .output()
                .await?;
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
    Ok(())
}

async fn handle_port_forward(
    namespace: String,
    service: String,
    local_port: u16,
    remote_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    // For port forwarding, we need to know which cluster to use
    // For now, we'll use the default cluster name
    let config = K3dClusterConfig::default();
    let manager = K3dClusterManager::new(config);
    
    let pf_config = PortForwardConfig {
        namespace,
        service_name: service.clone(),
        local_port,
        remote_port,
    };
    
    info!(
        "Setting up port forward: localhost:{} -> {}/{}:{}",
        local_port, pf_config.namespace, service, remote_port
    );
    
    manager.port_forward(&pf_config).await?;
    
    info!("Port forwarding established. Press Ctrl+C to stop.");
    
    // Keep the process running
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}

async fn handle_logs(
    resource_type: String,
    resource: String,
    namespace: Option<String>,
    container: Option<String>,
    follow: bool,
    previous: bool,
    all_containers: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = K3dClusterConfig::default();
    let manager = K3dClusterManager::new(config);
    
    let resource_type = match resource_type.as_str() {
        "pod" => ResourceType::Pod,
        "deployment" => ResourceType::Deployment,
        "service" => ResourceType::Service,
        "statefulset" => ResourceType::StatefulSet,
        "daemonset" => ResourceType::DaemonSet,
        "job" => ResourceType::Job,
        _ => {
            error!("Unknown resource type: {}", resource_type);
            return Ok(());
        }
    };
    
    let logs_config = LogsConfig {
        namespace,
        resource_type,
        resource_name: resource,
        container,
        follow,
        previous,
        all_containers,
    };
    
    if follow {
        info!("Streaming logs (press Ctrl+C to stop)...");
        manager.stream_logs(&logs_config).await?;
    } else {
        let logs = manager.get_logs(&logs_config).await?;
        println!("{}", logs);
    }
    
    Ok(())
}

async fn handle_diagnostics(
    namespace: Option<String>,
    nodes: bool,
    pods: bool,
    services: bool,
    events: bool,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = K3dClusterConfig::default();
    let manager = K3dClusterManager::new(config);
    
    let diag_config = DiagnosticsConfig {
        namespace,
        include_nodes: nodes || all,
        include_pods: pods || all,
        include_services: services || all,
        include_events: events || all,
        include_all: all,
    };
    
    info!("Getting cluster diagnostics...");
    let diagnostics = manager.get_diagnostics(&diag_config).await?;
    
    for (resource_type, content) in diagnostics {
        println!("\n=== {} ===", resource_type.to_uppercase());
        println!("{}", content);
    }
    
    Ok(())
}

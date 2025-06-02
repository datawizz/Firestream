//! Example of using k8s_manager for k3d cluster management

use k8s_manager::{
    K3dClusterManager, K3dClusterConfig, K3dDevModeConfig,
    ClusterManager, ClusterLifecycle, ClusterNetworking, 
    ClusterObservability, ClusterSecurity,
    DiagnosticsConfig, PortForwardConfig, LogsConfig, ResourceType,
};
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Example 1: Basic setup with convenience function
    basic_setup_example().await?;
    
    // Example 2: Advanced setup with full configuration
    // Uncomment the line below to run this example
    // advanced_setup_example().await?;
    
    // Example 3: Cluster management operations
    // Uncomment the line below to run this example
    // cluster_operations_example().await?;
    
    Ok(())
}

/// Example of basic cluster setup using convenience functions
async fn basic_setup_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Basic K3D Setup Example ===");
    
    // Use the simple setup function with defaults
    k8s_manager::k3d::setup_cluster().await?;
    
    info!("Basic cluster created with defaults!");
    info!("To delete: k3d cluster delete firestream");
    
    Ok(())
}

/// Example of advanced cluster setup with custom configuration
#[allow(dead_code)]
async fn advanced_setup_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Advanced K3D Setup Example ===");
    
    // Create k3d configuration
    let mut config = K3dClusterConfig::default();
    config.name = "example-cluster".to_string();
    config.agents = 2;
    config.api_port = 6551;
    
    // Enable development mode
    config.dev_mode = Some(K3dDevModeConfig {
        port_forward_all: true,
        port_offset: 20000,
    });
    
    // Customize TLS settings
    config.tls.enabled = true;
    config.tls.certificate_config.organization = "Example Corp".to_string();
    config.tls.certificate_config.common_name = "example.local".to_string();
    
    // Customize network settings
    config.network.configure_routes = true;
    config.network.configure_dns = true;

    // Create cluster manager
    let manager = K3dClusterManager::new(config);
    
    info!("Setting up k3d cluster...");
    
    // Full setup (includes registry, TLS, networking)
    manager.setup_cluster().await?;
    
    info!("Cluster setup complete!");
    
    // Get cluster information
    let info = manager.get_cluster_info().await?;
    println!("\nCluster Info:");
    println!("  Name: {}", info.name);
    println!("  Provider: {:?}", info.provider);
    println!("  Status: {:?}", info.status);
    println!("  Endpoint: {:?}", info.endpoint);
    println!("  K8s Version: {:?}", info.kubernetes_version);
    println!("  Node Count: {}", info.node_count);
    
    // Get diagnostics
    let diag_config = DiagnosticsConfig {
        include_nodes: true,
        include_pods: true,
        include_services: true,
        ..Default::default()
    };
    
    let diagnostics = manager.get_diagnostics(&diag_config).await?;
    
    for (key, value) in diagnostics {
        println!("\n=== {} ===", key);
        println!("{}", value);
    }
    
    info!("\nExample complete! Cluster is running.");
    info!("To delete: k3d cluster delete example-cluster");
    
    Ok(())
}

/// Example of various cluster operations
#[allow(dead_code)]
async fn cluster_operations_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Cluster Operations Example ===");
    
    let config = K3dClusterConfig {
        name: "ops-example".to_string(),
        ..Default::default()
    };
    
    let manager = K3dClusterManager::new(config);
    
    // Create cluster
    info!("Creating cluster...");
    manager.create_cluster().await?;
    
    // Wait for cluster to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    // Example: Port forwarding
    info!("\nSetting up port forwarding...");
    let pf_config = PortForwardConfig {
        namespace: "kube-system".to_string(),
        service_name: "kube-dns".to_string(),
        local_port: 15353,
        remote_port: 53,
    };
    manager.port_forward(&pf_config).await?;
    info!("Port forwarding established: localhost:15353 -> kube-dns:53");
    
    // Example: Create a secret
    info!("\nCreating a secret...");
    let mut secret_data = HashMap::new();
    secret_data.insert("username".to_string(), b"admin".to_vec());
    secret_data.insert("password".to_string(), b"secret123".to_vec());
    
    manager.create_secret(
        "example-secret",
        "default",
        secret_data
    ).await?;
    info!("Secret 'example-secret' created in 'default' namespace");
    
    // Example: Get logs
    info!("\nGetting logs from a system component...");
    let logs_config = LogsConfig {
        namespace: Some("kube-system".to_string()),
        resource_type: ResourceType::Deployment,
        resource_name: "coredns".to_string(),
        container: None,
        follow: false,
        previous: false,
        all_containers: true,
    };
    
    match manager.get_logs(&logs_config).await {
        Ok(logs) => {
            let preview = if logs.len() > 200 {
                &logs[..200]
            } else {
                &logs
            };
            println!("\nCoreDNS Logs (preview):\n{}", preview);
            if logs.len() > 200 {
                println!("... (truncated)");
            }
        }
        Err(e) => {
            println!("Failed to get logs: {}", e);
        }
    }
    
    // Example: Lifecycle operations
    info!("\nTesting lifecycle operations...");
    info!("Stopping cluster...");
    manager.stop_cluster().await?;
    
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    info!("Starting cluster...");
    manager.start_cluster().await?;
    
    info!("\nAll operations completed successfully!");
    info!("To delete: k3d cluster delete ops-example");
    
    Ok(())
}

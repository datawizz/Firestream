//! K3D cluster management
//!
//! This module handles K3D (K3s in Docker) cluster creation and management.

use crate::core::{FirestreamError, Result};
use tokio::process::Command;
use tracing::{info, warn};

/// K3D cluster configuration
pub struct K3dConfig {
    pub cluster_name: String,
    pub api_port: u16,
    pub lb_port: u16,
    pub agents: u32,
    pub servers: u32,
    pub registry_name: String,
    pub registry_port: u16,
}

impl Default for K3dConfig {
    fn default() -> Self {
        Self {
            cluster_name: "firestream".to_string(),
            api_port: 6443,
            lb_port: 8080,
            agents: 1,
            servers: 1,
            registry_name: "registry.localhost".to_string(),
            registry_port: 5000,
        }
    }
}

/// Setup K3D cluster
pub async fn setup_cluster() -> Result<()> {
    let config = K3dConfig::default();
    setup_cluster_with_config(&config).await
}

/// Setup K3D cluster with custom configuration
pub async fn setup_cluster_with_config(config: &K3dConfig) -> Result<()> {
    // Check if k3d is installed
    check_k3d_installed().await?;
    
    // Check if cluster already exists
    if cluster_exists(&config.cluster_name).await? {
        info!("K3D cluster '{}' already exists", config.cluster_name);
        return Ok(());
    }
    
    // Create registry if it doesn't exist
    create_registry(config).await?;
    
    // Create cluster
    create_cluster(config).await?;
    
    // Wait for cluster to be ready
    wait_for_cluster(&config.cluster_name).await?;
    
    // Configure kubectl context
    configure_kubectl(&config.cluster_name).await?;
    
    info!("K3D cluster '{}' is ready", config.cluster_name);
    Ok(())
}

/// Check if k3d is installed
async fn check_k3d_installed() -> Result<()> {
    match which::which("k3d") {
        Ok(path) => {
            info!("k3d found at: {:?}", path);
            
            // Get version
            match Command::new("k3d")
                .arg("version")
                .output()
                .await
            {
                Ok(output) if output.status.success() => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    info!("k3d version: {}", version.trim());
                    Ok(())
                }
                _ => Err(FirestreamError::GeneralError(
                    "Failed to get k3d version".to_string()
                )),
            }
        }
        Err(_) => Err(FirestreamError::GeneralError(
            "k3d is not installed. Please install k3d first.".to_string()
        )),
    }
}

/// Check if cluster exists
async fn cluster_exists(cluster_name: &str) -> Result<bool> {
    let output = Command::new("k3d")
        .args(&["cluster", "list", "-o", "json"])
        .output()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to list clusters: {}", e)))?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(&format!("\"name\":\"{}\"", cluster_name)))
    } else {
        Ok(false)
    }
}

/// Create Docker registry
async fn create_registry(config: &K3dConfig) -> Result<()> {
    // Check if registry exists
    let output = Command::new("docker")
        .args(&["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .await?;
    
    let containers = String::from_utf8_lossy(&output.stdout);
    if containers.lines().any(|name| name == config.registry_name) {
        info!("Registry '{}' already exists", config.registry_name);
        return Ok(());
    }
    
    info!("Creating Docker registry '{}'", config.registry_name);
    
    let status = Command::new("k3d")
        .args(&[
            "registry",
            "create",
            &config.registry_name,
            "--port",
            &config.registry_port.to_string(),
        ])
        .status()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to create registry: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            "Failed to create Docker registry".to_string()
        ));
    }
    
    Ok(())
}

/// Create K3D cluster
async fn create_cluster(config: &K3dConfig) -> Result<()> {
    info!("Creating K3D cluster '{}'", config.cluster_name);
    
    let mut cmd = Command::new("k3d");
    cmd.args(&[
        "cluster",
        "create",
        &config.cluster_name,
        "--api-port",
        &config.api_port.to_string(),
        "-p",
        &format!("{}:80@loadbalancer", config.lb_port),
        "--agents",
        &config.agents.to_string(),
        "--servers",
        &config.servers.to_string(),
        "--registry-use",
        &format!("{}:{}", config.registry_name, config.registry_port),
        "--k3s-arg",
        "--disable=traefik@server:*",
        "--wait",
    ]);
    
    let status = cmd.status().await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to create cluster: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            "Failed to create K3D cluster".to_string()
        ));
    }
    
    Ok(())
}

/// Wait for cluster to be ready
async fn wait_for_cluster(cluster_name: &str) -> Result<()> {
    info!("Waiting for cluster '{}' to be ready...", cluster_name);
    
    // Wait for nodes to be ready
    for i in 0..30 {
        let output = Command::new("kubectl")
            .args(&["get", "nodes", "-o", "jsonpath={.items[*].status.conditions[?(@.type=='Ready')].status}"])
            .output()
            .await?;
        
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout);
            if status.split_whitespace().all(|s| s == "True") {
                info!("All nodes are ready");
                return Ok(());
            }
        }
        
        if i < 29 {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
    
    Err(FirestreamError::GeneralError(
        "Timeout waiting for cluster to be ready".to_string()
    ))
}

/// Configure kubectl to use the cluster
async fn configure_kubectl(cluster_name: &str) -> Result<()> {
    let status = Command::new("k3d")
        .args(&["kubeconfig", "merge", cluster_name])
        .status()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to configure kubectl: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            "Failed to configure kubectl".to_string()
        ));
    }
    
    // Set as current context
    let status = Command::new("kubectl")
        .args(&["config", "use-context", &format!("k3d-{}", cluster_name)])
        .status()
        .await?;
    
    if !status.success() {
        warn!("Failed to set kubectl context");
    }
    
    Ok(())
}

/// Delete K3D cluster
pub async fn delete_cluster(cluster_name: &str) -> Result<()> {
    info!("Deleting K3D cluster '{}'", cluster_name);
    
    let status = Command::new("k3d")
        .args(&["cluster", "delete", cluster_name])
        .status()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to delete cluster: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            "Failed to delete K3D cluster".to_string()
        ));
    }
    
    Ok(())
}

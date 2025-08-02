//! Registry management for K3D clusters

use crate::{K8sManagerError, Result, K3dConfig, K3dClusterConfig, K3dRegistryConfig};
use crate::providers::k3d::manager::K3dClusterManager;
use tokio::process::Command;
use tokio::time::{sleep, Duration, timeout};
use tracing::{info, debug};

impl K3dClusterManager {
    /// Verify registry is healthy
    pub(crate) async fn verify_registry_health(&self, registry_name: &str) -> Result<()> {
        let registry_name = registry_name.to_string();
        let registry_port = self.config.registry.port;
        let cluster_name = self.config.name.clone();
        
        self.retry_with_backoff("registry health check", move || {
            let registry_name = registry_name.clone();
            let cluster_name = cluster_name.clone();
            async move {
                // First check if the container is running
                let full_name = if registry_name.starts_with("k3d-") {
                    registry_name.to_string()
                } else {
                    format!("k3d-{}", registry_name)
                };
                
                let container_check = Command::new("docker")
                    .args(&[
                        "ps",
                        "--filter", &format!("name={}", full_name),
                        "--filter", "status=running",
                        "--format", "{{.Names}}"
                    ])
                    .output()
                    .await?;
                    
                if !String::from_utf8_lossy(&container_check.stdout).contains(&full_name) {
                    return Err(K8sManagerError::GeneralError(
                        format!("Registry container {} is not running", full_name)
                    ));
                }
                
                // Determine the registry URL based on environment
                let registry_url = if crate::providers::k3d::utils::is_running_in_container() {
                    // When in a container, we need to get the registry's IP address
                    // because DNS resolution might not work due to custom resolv.conf
                    let inspect_output = Command::new("docker")
                        .args(&[
                            "inspect",
                            &full_name,
                            "--format",
                            "{{json .NetworkSettings.Networks}}"
                        ])
                        .output()
                        .await?;
                    
                    if inspect_output.status.success() {
                        // Parse JSON to find the IP on the k3d network
                        let networks_json = String::from_utf8_lossy(&inspect_output.stdout);
                        if let Ok(networks) = serde_json::from_str::<serde_json::Value>(&networks_json) {
                            // Look for the k3d network
                            let k3d_network = format!("k3d-{}", cluster_name);
                            if let Some(network_info) = networks.get(&k3d_network) {
                                if let Some(ip) = network_info.get("IPAddress").and_then(|v| v.as_str()) {
                                    if !ip.is_empty() {
                                        debug!("Using registry IP address on {}: {}", k3d_network, ip);
                                        format!("http://{}:5000/v2/", ip)
                                    } else {
                                        debug!("No IP found on k3d network, trying container name");
                                        format!("http://{}:5000/v2/", full_name)
                                    }
                                } else {
                                    debug!("No IPAddress field found, trying container name");
                                    format!("http://{}:5000/v2/", full_name)
                                }
                            } else {
                                debug!("Registry not on k3d network {}, trying container name", k3d_network);
                                format!("http://{}:5000/v2/", full_name)
                            }
                        } else {
                            debug!("Failed to parse network JSON, trying container name");
                            format!("http://{}:5000/v2/", full_name)
                        }
                    } else {
                        // Fallback to container name
                        debug!("Docker inspect failed, trying container name");
                        format!("http://{}:5000/v2/", full_name)
                    }
                } else {
                    // When not in a container, use localhost with the mapped port
                    format!("http://localhost:{}/v2/", registry_port)
                };
                
                debug!("Registry health check URL: {}", registry_url);
                
                // Then check HTTP endpoint
                let output = Command::new("curl")
                    .args(&[
                        "-s",
                        "-o", "/dev/null",
                        "-w", "%{http_code}",
                        "--connect-timeout", "5",
                        &registry_url
                    ])
                    .output()
                    .await?;
                
                if output.status.success() {
                    let status_code = String::from_utf8_lossy(&output.stdout);
                    debug!("Registry health check HTTP status: '{}'", status_code.trim());
                    if status_code.trim() == "200" {
                        debug!("Registry {} is healthy", registry_name);
                        return Ok(());
                    }
                    Err(K8sManagerError::GeneralError(
                        format!("Registry {} returned HTTP status: {}", registry_name, status_code.trim())
                    ))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    debug!("Curl command failed: {}", stderr);
                    Err(K8sManagerError::GeneralError(
                        format!("Registry {} is not responding properly (curl failed)", registry_name)
                    ))
                }
            }
        }).await
    }
    
    /// Wait for registry to be ready
    pub(crate) async fn wait_for_registry_ready(&self, registry_name: &str) -> Result<()> {
        info!("Waiting for registry '{}' to be ready...", registry_name);
        
        let timeout_duration = Duration::from_secs(self.config.timeouts.registry_ready);
        
        let result = timeout(timeout_duration, async {
            // Determine the full container name
            let full_name = if registry_name.starts_with("k3d-") {
                registry_name.to_string()
            } else {
                format!("k3d-{}", registry_name)
            };
            
            // First, wait for the container to be running
            self.retry_with_backoff("registry container check", || async {
                let output = Command::new("docker")
                    .args(&[
                        "ps",
                        "--filter", &format!("name={}", full_name),
                        "--filter", "status=running",
                        "--format", "{{.Names}}"
                    ])
                    .output()
                    .await?;
                
                if !output.status.success() {
                    return Err(K8sManagerError::GeneralError(
                        "Failed to check registry container".to_string()
                    ));
                }
                
                let containers = String::from_utf8_lossy(&output.stdout);
                if !containers.contains(&full_name) {
                    return Err(K8sManagerError::GeneralError(
                        format!("Registry container {} is not running", full_name)
                    ));
                }
                
                Ok(())
            }).await?;
            
            // Then verify it's actually responding
            self.verify_registry_health(registry_name).await
        }).await
        .map_err(|_| K8sManagerError::Timeout(
            format!("Registry {} failed to become ready within {:?}", registry_name, timeout_duration)
        ))?;
        
        result?;
        
        info!("Registry '{}' is ready", registry_name);
        Ok(())
    }
}

/// Create Docker registry
pub async fn create_registry(config: &K3dConfig) -> Result<()> {
    // Check if registry exists
    let output = Command::new("docker")
        .args(&["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .await?;
    
    let containers = String::from_utf8_lossy(&output.stdout);
    let full_registry_name = format!("k3d-{}", config.registry_name);
    
    if containers.lines().any(|name| name == full_registry_name || name == config.registry_name) {
        info!("Registry '{}' already exists", config.registry_name);
        
        // Create a temporary manager to verify registry health
        let temp_config = K3dClusterConfig {
            registry: K3dRegistryConfig {
                name: config.registry_name.clone(),
                port: config.registry_port,
                ..Default::default()
            },
            ..Default::default()
        };
        let manager = K3dClusterManager::new(temp_config);
        return manager.verify_registry_health(&config.registry_name).await;
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
        .map_err(|e| K8sManagerError::ProcessError(format!("Failed to create registry: {}", e)))?;
    
    if !status.success() {
        return Err(K8sManagerError::GeneralError(
            "Failed to create Docker registry".to_string()
        ));
    }
    
    // Wait a moment for the registry to start
    sleep(Duration::from_secs(2)).await;
    
    // Wait for registry to be ready
    let temp_config = K3dClusterConfig {
        registry: K3dRegistryConfig {
            port: config.registry_port,
            ..Default::default()
        },
        ..Default::default()
    };
    let manager = K3dClusterManager::new(temp_config);
    manager.wait_for_registry_ready(&config.registry_name).await?;
    
    Ok(())
}
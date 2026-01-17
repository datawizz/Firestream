//! Development features for K3D clusters

use crate::{K8sManagerError, Result};
use crate::providers::k3d::manager::K3dClusterManager;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tracing::{info};

impl K3dClusterManager {
    /// Enable development mode
    pub async fn enable_dev_mode(&self) -> Result<()> {
        if let Some(dev_config) = &self.config.dev_mode {
            if dev_config.port_forward_all {
                self.setup_port_forwarding(dev_config).await?;
            }
        }
        Ok(())
    }
    
    /// Disable development mode
    pub async fn disable_dev_mode(&self) -> Result<()> {
        // Kill all kubectl port-forward processes
        let _ = Command::new("pkill")
            .args(&["-f", "kubectl port-forward"])
            .status()
            .await;
        
        Ok(())
    }
    
    /// Setup local registry
    pub async fn setup_registry(&self) -> Result<()> {
        // Use the registry name from config if it's cluster-specific, otherwise use cluster-registry pattern
        let registry_name = if self.config.registry.name.contains(&self.config.name) {
            self.config.registry.name.clone()
        } else if self.config.registry.name == "registry.localhost" {
            format!("{}-registry", self.config.name)
        } else {
            self.config.registry.name.clone()
        };
        
        // Check if registry exists by looking at Docker containers
        let output = Command::new("docker")
            .args(&["ps", "-a", "--format", "{{.Names}}"])
            .output()
            .await?;
        
        let containers = String::from_utf8_lossy(&output.stdout);
        let full_registry_name = format!("k3d-{}", registry_name);
        
        if containers.lines().any(|name| name == full_registry_name) {
            info!("Registry '{}' already exists", registry_name);
            
            // Check if it's running
            let running_check = Command::new("docker")
                .args(&[
                    "ps",
                    "--filter", &format!("name={}", full_registry_name),
                    "--filter", "status=running",
                    "--format", "{{.Names}}"
                ])
                .output()
                .await?;
                
            if !String::from_utf8_lossy(&running_check.stdout).contains(&full_registry_name) {
                // Start the registry if it's not running
                info!("Starting existing registry '{}'", registry_name);
                let _ = Command::new("docker")
                    .args(&["start", &full_registry_name])
                    .status()
                    .await?;
                sleep(Duration::from_secs(2)).await;
            }
            
            // If we're in a container, ensure we're connected to the cluster network  
            if crate::providers::k3d::utils::is_running_in_container() {
                let cluster_network = format!("k3d-{}", self.config.name);
                let network_check = Command::new("docker")
                    .args(&["network", "ls", "--format", "{{.Name}}"])
                    .output()
                    .await?;
                
                if String::from_utf8_lossy(&network_check.stdout)
                    .lines()
                    .any(|line| line == cluster_network) {
                    info!("Connecting devcontainer to cluster network for registry access");
                    self.connect_to_cluster_network().await?;
                }
            }
            
            return self.verify_registry_health(&registry_name).await;
        }
        
        info!("Creating registry '{}' on port {}", registry_name, self.config.registry.port);
        
        // Create registry with k3d command
        // Use the cluster network if it exists, otherwise use bridge
        let cluster_network = format!("k3d-{}", self.config.name);
        
        // Check if the cluster network exists
        let network_check = Command::new("docker")
            .args(&["network", "ls", "--format", "{{.Name}}"])
            .output()
            .await?;
        
        let network_exists = String::from_utf8_lossy(&network_check.stdout)
            .lines()
            .any(|line| line == cluster_network);
        
        let mut cmd = Command::new("k3d");
        cmd.args(&[
            "registry",
            "create",
            &registry_name,
            "--port",
            &self.config.registry.port.to_string(),
        ]);
        
        if network_exists {
            cmd.args(&["--default-network", &cluster_network]);
            info!("Creating registry on cluster network: {}", cluster_network);
        } else {
            info!("Creating registry on default bridge network (cluster network doesn't exist yet)");
        }
        
        let status = cmd.status()
            .await
            .map_err(|e| K8sManagerError::ProcessError(format!("Failed to create registry: {}", e)))?;
        
        if !status.success() {
            return Err(K8sManagerError::GeneralError("Failed to create registry".to_string()));
        }
        
        // Wait for registry to be ready
        sleep(Duration::from_secs(2)).await;
        
        // If we're in a container and the cluster network exists, ensure we're connected to it
        if network_exists && crate::providers::k3d::utils::is_running_in_container() {
            info!("Connecting devcontainer to cluster network for registry access");
            self.connect_to_cluster_network().await?;
        }
        
        self.wait_for_registry_ready(&registry_name).await?;
        
        Ok(())
    }
}
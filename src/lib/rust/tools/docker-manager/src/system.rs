//! System-level Docker operations

use crate::{DockerManager, Result};
use bollard::models::{SystemInfo, SystemDataUsageResponse};
use bollard::system::Version as SystemVersion;
use tracing::info;

/// System operations
impl DockerManager {
    /// Get Docker version information
    pub async fn version(&self) -> Result<SystemVersion> {
        let version = self.client.version().await?;
        Ok(version)
    }
    
    /// Get Docker system information
    pub async fn info(&self) -> Result<SystemInfo> {
        let info = self.client.info().await?;
        Ok(info)
    }
    
    /// Get disk usage information
    pub async fn disk_usage(&self) -> Result<SystemDataUsageResponse> {
        let usage = self.client.df().await?;
        Ok(usage)
    }
    
    /// Ping the Docker daemon
    pub async fn ping(&self) -> Result<String> {
        let response = self.client.ping().await?;
        Ok(response)
    }
    
    /// Get system events
    pub async fn events(&self, since: Option<i64>, until: Option<i64>) -> Result<impl futures_util::Stream<Item = Result<bollard::models::EventMessage>>> {
        use bollard::system::EventsOptions;
        use futures_util::stream::TryStreamExt;
        
        let options = EventsOptions::<String> {
            since: since.map(|s| s.to_string()),
            until: until.map(|u| u.to_string()),
            ..Default::default()
        };
        
        let stream = self.client
            .events(Some(options))
            .map_err(|e| e.into());
        
        Ok(stream)
    }
    
    /// Prune all unused resources
    pub async fn prune_all(&self) -> Result<PruneAllResult> {
        info!("Pruning all unused Docker resources...");
        
        // Prune containers
        let containers_result = self.client.prune_containers::<String>(None).await?;
        let containers_deleted = containers_result.containers_deleted.unwrap_or_default().len();
        let containers_space = containers_result.space_reclaimed.unwrap_or(0) as u64;
        
        // Prune images
        let (images_space, images_deleted) = self.prune_images(false).await?;
        
        // Prune volumes
        let (volumes_space, volumes_deleted) = self.prune_volumes().await?;
        
        // Prune networks
        let networks_deleted = self.prune_networks().await?;
        
        let result = PruneAllResult {
            containers_deleted,
            containers_space_reclaimed: containers_space,
            images_deleted: images_deleted.len(),
            images_space_reclaimed: images_space,
            volumes_deleted: volumes_deleted.len(),
            volumes_space_reclaimed: volumes_space,
            networks_deleted: networks_deleted.len(),
            total_space_reclaimed: containers_space + images_space + volumes_space,
        };
        
        info!("Prune complete: {} total bytes reclaimed", result.total_space_reclaimed);
        Ok(result)
    }
    
    /// Get Docker daemon configuration
    pub async fn get_daemon_config(&self) -> Result<String> {
        // This would typically read from /etc/docker/daemon.json or similar
        // For now, we'll return the info from system info
        let info = self.info().await?;
        Ok(serde_json::to_string_pretty(&info)?)
    }
    
    /// Check Docker system health
    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        let mut result = HealthCheckResult::default();
        
        // Check if daemon is responsive
        match self.ping().await {
            Ok(_) => {
                result.daemon_responsive = true;
            }
            Err(e) => {
                result.errors.push(format!("Daemon not responsive: {}", e));
            }
        }
        
        // Check system info
        match self.info().await {
            Ok(info) => {
                result.total_containers = info.containers.unwrap_or(0) as usize;
                result.running_containers = info.containers_running.unwrap_or(0) as usize;
                result.total_images = info.images.unwrap_or(0) as usize;
                
                // Check for warnings
                if let Some(warnings) = info.warnings {
                    result.warnings.extend(warnings);
                }
            }
            Err(e) => {
                result.errors.push(format!("Failed to get system info: {}", e));
            }
        }
        
        // Check disk usage
        match self.disk_usage().await {
            Ok(usage) => {
                if let Some(images) = usage.images {
                    let total_size: i64 = images.iter()
                        .map(|img| img.size)
                        .sum();
                    result.disk_usage_bytes = total_size as u64;
                }
            }
            Err(e) => {
                result.warnings.push(format!("Failed to get disk usage: {}", e));
            }
        }
        
        result.healthy = result.daemon_responsive && result.errors.is_empty();
        Ok(result)
    }
}

/// Result of pruning all resources
#[derive(Debug, Clone)]
pub struct PruneAllResult {
    pub containers_deleted: usize,
    pub containers_space_reclaimed: u64,
    pub images_deleted: usize,
    pub images_space_reclaimed: u64,
    pub volumes_deleted: usize,
    pub volumes_space_reclaimed: u64,
    pub networks_deleted: usize,
    pub total_space_reclaimed: u64,
}

/// Docker system health check result
#[derive(Debug, Default, Clone)]
pub struct HealthCheckResult {
    pub healthy: bool,
    pub daemon_responsive: bool,
    pub total_containers: usize,
    pub running_containers: usize,
    pub total_images: usize,
    pub disk_usage_bytes: u64,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}
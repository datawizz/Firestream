//! Volume management operations

use crate::{DockerManager, DockerManagerError, Result};
use bollard::volume::{CreateVolumeOptions, ListVolumesOptions, RemoveVolumeOptions};
use bollard::models::{Volume, VolumeListResponse};
use std::collections::HashMap;
use tracing::{debug, info};
use futures_util::StreamExt;

/// Volume operations
impl DockerManager {
    /// List all volumes
    pub async fn list_volumes(&self) -> Result<Vec<Volume>> {
        let options = ListVolumesOptions::<String> {
            ..Default::default()
        };
        
        let response: VolumeListResponse = self.client.list_volumes(Some(options)).await?;
        let volumes = response.volumes.unwrap_or_default();
        debug!("Found {} volumes", volumes.len());
        Ok(volumes)
    }
    
    /// Get volume by name
    pub async fn get_volume(&self, name: &str) -> Result<Volume> {
        let volumes = self.list_volumes().await?;
        
        volumes
            .into_iter()
            .find(|v| v.name == name)
            .ok_or_else(|| DockerManagerError::VolumeNotFound(name.to_string()))
    }
    
    /// Create a new volume
    pub async fn create_volume(
        &self,
        name: &str,
        driver: Option<&str>,
        labels: Option<HashMap<String, String>>,
    ) -> Result<Volume> {
        let options = CreateVolumeOptions {
            name: name.to_string(),
            driver: driver.unwrap_or("local").to_string(),
            labels: labels.unwrap_or_default(),
            driver_opts: HashMap::new(),
        };
        
        let volume = self.client.create_volume(options).await?;
        info!("Created volume '{}'", name);
        Ok(volume)
    }
    
    /// Remove a volume
    pub async fn remove_volume(&self, name: &str, force: bool) -> Result<()> {
        let options = RemoveVolumeOptions {
            force,
        };
        
        self.client.remove_volume(name, Some(options)).await?;
        info!("Removed volume '{}'", name);
        Ok(())
    }
    
    /// Inspect a volume
    pub async fn inspect_volume(&self, name: &str) -> Result<Volume> {
        let volume = self.client.inspect_volume(name).await?;
        Ok(volume)
    }
    
    /// Prune unused volumes
    pub async fn prune_volumes(&self) -> Result<(u64, Vec<String>)> {
        let result = self.client.prune_volumes::<String>(None).await?;
        
        let space_reclaimed = result.space_reclaimed.unwrap_or(0) as u64;
        let volumes_deleted = result.volumes_deleted.unwrap_or_default();
        
        info!("Pruned {} volumes: {} bytes reclaimed", volumes_deleted.len(), space_reclaimed);
        Ok((space_reclaimed, volumes_deleted))
    }
    
    /// Check if volume exists
    pub async fn volume_exists(&self, name: &str) -> bool {
        self.get_volume(name).await.is_ok()
    }
    
    /// Get volumes used by a container
    pub async fn get_container_volumes(&self, container_name_or_id: &str) -> Result<Vec<String>> {
        let inspect = self.client.inspect_container(container_name_or_id, None).await?;
        
        let volumes = inspect
            .mounts
            .unwrap_or_default()
            .into_iter()
            .filter_map(|mount| mount.name)
            .collect();
        
        Ok(volumes)
    }
    
    /// Copy data between volumes
    pub async fn copy_volume_data(&self, source_volume: &str, dest_volume: &str) -> Result<()> {
        // This is a helper function that uses a temporary container to copy data
        let config = bollard::container::Config {
            image: Some("busybox:latest".to_string()),
            cmd: Some(vec!["sh".to_string(), "-c".to_string(), 
                format!("cp -a /source/. /dest/")]),
            host_config: Some(bollard::models::HostConfig {
                binds: Some(vec![
                    format!("{}:/source:ro", source_volume),
                    format!("{}:/dest", dest_volume),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        let temp_container = format!("volume-copy-{}", chrono::Utc::now().timestamp());
        let id = self.create_container(&temp_container, "busybox:latest", Some(config)).await?;
        
        // Start the container
        self.start_container(&id).await?;
        
        // Wait for it to complete
        let mut wait_stream = self.client.wait_container::<String>(&id, None);
        let _ = wait_stream.next().await;
        
        // Remove the container
        self.remove_container(&id, true).await?;
        
        info!("Copied data from volume '{}' to '{}'", source_volume, dest_volume);
        Ok(())
    }
}
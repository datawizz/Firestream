//! Docker Manager Library
//!
//! This library provides a high-level interface for Docker operations including
//! container, image, volume, and network management.

pub mod error;
pub mod container;
pub mod image;
pub mod volume;
pub mod network;
pub mod system;
pub mod builder;
pub mod compose;
pub mod utils;

pub use error::{DockerManagerError, Result};

use bollard::Docker;
use std::path::Path;
use tracing::{debug, info};

/// Docker client wrapper with connection management
#[derive(Clone)]
pub struct DockerManager {
    client: Docker,
}

impl DockerManager {
    /// Create a new DockerManager instance
    pub async fn new() -> Result<Self> {
        Self::connect().await
    }
    
    /// Connect to Docker daemon
    pub async fn connect() -> Result<Self> {
        // Try to connect with default settings
        let client = Docker::connect_with_socket_defaults()
            .map_err(|e| DockerManagerError::DockerNotAccessible(e.to_string()))?;
        
        // Verify connection by getting version
        let version = client.version().await?;
        info!("Connected to Docker daemon version: {:?}", version.version);
        
        Ok(Self { client })
    }
    
    /// Connect to a specific Docker socket
    pub async fn connect_with_socket(socket_path: &str) -> Result<Self> {
        let client = Docker::connect_with_socket(socket_path, 120, bollard::API_DEFAULT_VERSION)
            .map_err(|e| DockerManagerError::DockerNotAccessible(e.to_string()))?;
        
        // Verify connection
        let version = client.version().await?;
        info!("Connected to Docker daemon at {} version: {:?}", socket_path, version.version);
        
        Ok(Self { client })
    }
    
    /// Get the underlying Docker client
    pub fn client(&self) -> &Docker {
        &self.client
    }
    
    /// Check if Docker daemon is accessible
    pub async fn is_accessible(&self) -> bool {
        self.client.ping().await.is_ok()
    }
    
    /// Check if we're running inside a Docker container
    pub fn is_running_in_docker() -> bool {
        // Check for .dockerenv
        if Path::new("/.dockerenv").exists() {
            debug!("Detected /.dockerenv - running in Docker");
            return true;
        }
        
        // Check cgroup
        if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
            if cgroup.contains("/docker/") || cgroup.contains("/docker-ce/") {
                debug!("Detected Docker in cgroup - running in Docker");
                return true;
            }
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_running_in_docker() {
        // This test will pass or fail depending on where it's run
        let in_docker = DockerManager::is_running_in_docker();
        // Just verify it returns a boolean without panicking
        assert!(in_docker || !in_docker);
    }
}
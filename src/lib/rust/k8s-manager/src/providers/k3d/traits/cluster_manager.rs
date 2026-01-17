//! ClusterManager trait implementation for K3D

use async_trait::async_trait;
use crate::{Result, ClusterInfo};
use crate::traits::ClusterManager;
use crate::providers::k3d::manager::K3dClusterManager;

#[async_trait]
impl ClusterManager for K3dClusterManager {
    fn provider_name(&self) -> &'static str {
        "k3d"
    }
    
    async fn create_cluster(&self) -> Result<()> {
        self.create_cluster().await
    }
    
    async fn delete_cluster(&self) -> Result<()> {
        self.delete_cluster().await
    }
    
    async fn cluster_exists(&self) -> Result<bool> {
        self.cluster_exists().await
    }
    
    async fn get_cluster_info(&self) -> Result<ClusterInfo> {
        self.get_cluster_info().await
    }
    
    async fn get_cluster_status(&self) -> Result<String> {
        self.get_cluster_status().await
    }
    
    async fn connect_cluster(&self) -> Result<()> {
        self.connect_cluster().await
    }
    
    async fn update_cluster(&self) -> Result<()> {
        // K3D doesn't support in-place updates, would need to recreate
        Err(crate::K8sManagerError::GeneralError(
            "K3D clusters cannot be updated in-place. Please delete and recreate.".to_string()
        ))
    }
}
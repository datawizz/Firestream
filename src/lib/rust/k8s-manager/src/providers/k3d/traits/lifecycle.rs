//! ClusterLifecycle trait implementation for K3D

use async_trait::async_trait;
use crate::Result;
use crate::traits::ClusterLifecycle;
use crate::providers::k3d::manager::K3dClusterManager;

#[async_trait]
impl ClusterLifecycle for K3dClusterManager {
    async fn start_cluster(&self) -> Result<()> {
        self.start_cluster().await
    }
    
    async fn stop_cluster(&self) -> Result<()> {
        self.stop_cluster().await
    }
}
//! ClusterDevelopment trait implementation for K3D

use async_trait::async_trait;
use crate::Result;
use crate::traits::ClusterDevelopment;
use crate::providers::k3d::manager::K3dClusterManager;

#[async_trait]
impl ClusterDevelopment for K3dClusterManager {
    async fn enable_dev_mode(&self) -> Result<()> {
        self.enable_dev_mode().await
    }
    
    async fn disable_dev_mode(&self) -> Result<()> {
        self.disable_dev_mode().await
    }
    
    async fn setup_registry(&self) -> Result<()> {
        self.setup_registry().await
    }
}
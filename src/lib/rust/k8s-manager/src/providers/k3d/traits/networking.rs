//! ClusterNetworking trait implementation for K3D

use async_trait::async_trait;
use crate::{Result, PortForwardConfig};
use crate::traits::ClusterNetworking;
use crate::providers::k3d::manager::K3dClusterManager;

#[async_trait]
impl ClusterNetworking for K3dClusterManager {
    async fn port_forward(&self, config: &PortForwardConfig) -> Result<()> {
        self.port_forward(config).await
    }
    
    async fn port_forward_all(&self, port_offset: u16) -> Result<Vec<PortForwardConfig>> {
        self.port_forward_all(port_offset).await
    }
    
    async fn configure_dns(&self) -> Result<()> {
        self.configure_dns().await
    }
    
    async fn setup_routes(&self) -> Result<()> {
        self.setup_routes().await
    }
}
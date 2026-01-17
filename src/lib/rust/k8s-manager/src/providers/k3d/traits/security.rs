//! ClusterSecurity trait implementation for K3D

use async_trait::async_trait;
use crate::Result;
use crate::traits::ClusterSecurity;
use crate::providers::k3d::manager::K3dClusterManager;
use std::collections::HashMap;

#[async_trait]
impl ClusterSecurity for K3dClusterManager {
    async fn configure_tls(&self) -> Result<()> {
        self.configure_tls().await
    }
    
    async fn create_secret(&self, name: &str, namespace: &str, data: HashMap<String, Vec<u8>>) -> Result<()> {
        self.create_secret(name, namespace, data).await
    }
    
    async fn get_secret(&self, name: &str, namespace: &str) -> Result<HashMap<String, Vec<u8>>> {
        self.get_secret(name, namespace).await
    }
}
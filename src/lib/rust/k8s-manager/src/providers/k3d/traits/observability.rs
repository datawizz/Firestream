//! ClusterObservability trait implementation for K3D

use async_trait::async_trait;
use crate::{Result, LogsConfig, DiagnosticsConfig};
use crate::traits::ClusterObservability;
use crate::providers::k3d::manager::K3dClusterManager;
use std::collections::HashMap;

#[async_trait]
impl ClusterObservability for K3dClusterManager {
    async fn get_logs(&self, config: &LogsConfig) -> Result<String> {
        self.get_logs(config).await
    }
    
    async fn stream_logs(&self, config: &LogsConfig) -> Result<()> {
        self.stream_logs(config).await
    }
    
    async fn get_diagnostics(&self, config: &DiagnosticsConfig) -> Result<HashMap<String, String>> {
        self.get_diagnostics(config).await
    }
    
    async fn get_metrics(&self) -> Result<HashMap<String, serde_json::Value>> {
        self.get_metrics().await
    }
}
//! Traits for Kubernetes cluster management

use async_trait::async_trait;
use crate::{Result, ClusterInfo, PortForwardConfig, LogsConfig, DiagnosticsConfig};
use std::collections::HashMap;

/// Main trait for Kubernetes cluster managers
#[async_trait]
pub trait ClusterManager: Send + Sync {
    /// Get cluster provider name
    fn provider_name(&self) -> &'static str;
    
    /// Create a new cluster
    async fn create_cluster(&self) -> Result<()>;
    
    /// Delete a cluster
    async fn delete_cluster(&self) -> Result<()>;
    
    /// Check if cluster exists
    async fn cluster_exists(&self) -> Result<bool>;
    
    /// Get cluster information
    async fn get_cluster_info(&self) -> Result<ClusterInfo>;
    
    /// Get cluster status
    async fn get_cluster_status(&self) -> Result<String>;
    
    /// Connect to an existing cluster
    async fn connect_cluster(&self) -> Result<()>;
    
    /// Update cluster configuration
    async fn update_cluster(&self) -> Result<()>;
}

/// Trait for cluster lifecycle management
#[async_trait]
pub trait ClusterLifecycle: ClusterManager {
    /// Start a stopped cluster
    async fn start_cluster(&self) -> Result<()>;
    
    /// Stop a running cluster
    async fn stop_cluster(&self) -> Result<()>;
    
    /// Restart the cluster
    async fn restart_cluster(&self) -> Result<()> {
        self.stop_cluster().await?;
        self.start_cluster().await
    }
}

/// Trait for cluster networking features
#[async_trait]
pub trait ClusterNetworking: ClusterManager {
    /// Setup port forwarding
    async fn port_forward(&self, config: &PortForwardConfig) -> Result<()>;
    
    /// Setup port forwarding for all services
    async fn port_forward_all(&self, port_offset: u16) -> Result<Vec<PortForwardConfig>>;
    
    /// Configure DNS settings
    async fn configure_dns(&self) -> Result<()>;
    
    /// Setup network routes
    async fn setup_routes(&self) -> Result<()>;
}

/// Trait for cluster observability
#[async_trait]
pub trait ClusterObservability: ClusterManager {
    /// Get logs from resources
    async fn get_logs(&self, config: &LogsConfig) -> Result<String>;
    
    /// Stream logs from resources
    async fn stream_logs(&self, config: &LogsConfig) -> Result<()>;
    
    /// Get cluster diagnostics
    async fn get_diagnostics(&self, config: &DiagnosticsConfig) -> Result<HashMap<String, String>>;
    
    /// Get cluster metrics
    async fn get_metrics(&self) -> Result<HashMap<String, serde_json::Value>>;
}

/// Trait for cluster security features
#[async_trait]
pub trait ClusterSecurity: ClusterManager {
    /// Configure TLS certificates
    async fn configure_tls(&self) -> Result<()>;
    
    /// Create or update secrets
    async fn create_secret(&self, name: &str, namespace: &str, data: HashMap<String, Vec<u8>>) -> Result<()>;
    
    /// Get secret
    async fn get_secret(&self, name: &str, namespace: &str) -> Result<HashMap<String, Vec<u8>>>;
}

/// Trait for development features
#[async_trait]
pub trait ClusterDevelopment: ClusterManager {
    /// Enable development mode
    async fn enable_dev_mode(&self) -> Result<()>;
    
    /// Disable development mode
    async fn disable_dev_mode(&self) -> Result<()>;
    
    /// Setup local registry
    async fn setup_registry(&self) -> Result<()>;
}

/// Combined trait for full-featured cluster managers
pub trait FullClusterManager: 
    ClusterManager + 
    ClusterLifecycle + 
    ClusterNetworking + 
    ClusterObservability + 
    ClusterSecurity + 
    ClusterDevelopment 
{}

// Blanket implementation
impl<T> FullClusterManager for T 
where 
    T: ClusterManager + 
       ClusterLifecycle + 
       ClusterNetworking + 
       ClusterObservability + 
       ClusterSecurity + 
       ClusterDevelopment 
{}

//! Core K3D cluster manager implementation

use crate::{K3dClusterConfig, Result};
use std::path::PathBuf;
use tokio::time::Duration;

/// K3D cluster manager with comprehensive features
pub struct K3dClusterManager {
    pub(crate) config: K3dClusterConfig,
    pub(crate) project_root: PathBuf,
}

impl K3dClusterManager {
    /// Create a new K3D cluster manager
    pub fn new(config: K3dClusterConfig) -> Self {
        Self {
            config,
            project_root: std::env::current_dir().unwrap_or_default(),
        }
    }
    
    /// Calculate exponential backoff delay
    pub(crate) fn calculate_backoff(&self, attempt: u32) -> Duration {
        let base_delay = self.config.timeouts.initial_retry_delay_ms;
        let max_delay = self.config.timeouts.max_retry_delay_ms;
        let delay = base_delay * 2u64.pow(attempt);
        Duration::from_millis(delay.min(max_delay))
    }
    
    /// Execute a function with retries and exponential backoff
    pub(crate) async fn retry_with_backoff<F, Fut, T>(&self, operation_name: &str, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        crate::providers::k3d::utils::retry_with_backoff(
            operation_name,
            self.config.timeouts.max_retries,
            self.config.timeouts.initial_retry_delay_ms,
            self.config.timeouts.max_retry_delay_ms,
            f,
        ).await
    }
}
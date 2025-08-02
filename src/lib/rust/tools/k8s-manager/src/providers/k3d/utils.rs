//! Utility functions for K3D cluster management

use crate::{K8sManagerError, Result};
use tokio::time::{sleep, Duration, timeout};
use tracing::{info, warn, error};
use std::future::Future;

/// Calculate exponential backoff delay
pub fn calculate_backoff(initial_delay_ms: u64, max_delay_ms: u64, attempt: u32) -> Duration {
    let delay = initial_delay_ms * 2u64.pow(attempt);
    Duration::from_millis(delay.min(max_delay_ms))
}

/// Execute a function with retries and exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    operation_name: &str,
    max_retries: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    mut f: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut last_error = None;
    
    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => {
                if attempt > 0 {
                    info!("{} succeeded after {} retries", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries - 1 {
                    let delay = calculate_backoff(initial_delay_ms, max_delay_ms, attempt);
                    warn!(
                        "{} attempt {} failed, retrying in {:?}: {}",
                        operation_name,
                        attempt + 1,
                        delay,
                        last_error.as_ref().unwrap()
                    );
                    sleep(delay).await;
                }
            }
        }
    }
    
    error!("{} failed after {} attempts", operation_name, max_retries);
    Err(last_error.unwrap_or_else(|| {
        K8sManagerError::GeneralError(format!("{} failed after {} attempts", operation_name, max_retries))
    }))
}

/// Execute a function with timeout and retries
pub async fn retry_with_timeout<F, Fut, T>(
    operation_name: &str,
    timeout_duration: Duration,
    max_retries: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    f: F,
) -> Result<T>
where
    F: FnMut() -> Fut + Clone,
    Fut: Future<Output = Result<T>>,
{
    timeout(timeout_duration, async move {
        retry_with_backoff(
            operation_name,
            max_retries,
            initial_delay_ms,
            max_delay_ms,
            f,
        ).await
    }).await
    .map_err(|_| K8sManagerError::Timeout(format!("{} timed out after {:?}", operation_name, timeout_duration)))?
}

/// Detect if we're running inside a container
pub fn is_running_in_container() -> bool {
    // Check multiple indicators for container environment
    if std::path::Path::new("/.dockerenv").exists() {
        return true;
    }
    
    // Check if running in Kubernetes pod
    if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        return true;
    }
    
    // Check common container environment variables
    if std::env::var("CONTAINER_REGISTRY_URL").is_ok() || 
       std::env::var("CONTAINER").is_ok() ||
       std::env::var("container").is_ok() {
        return true;
    }
    
    // Check /proc/1/cgroup for container signatures
    if let Ok(contents) = std::fs::read_to_string("/proc/1/cgroup") {
        if contents.contains("docker") || 
           contents.contains("containerd") || 
           contents.contains("kubepods") ||
           contents.contains("/lxc/") {
            return true;
        }
    }
    
    false
}

/// Get the appropriate API endpoint based on context
pub fn get_api_endpoint(_cluster_name: &str, api_port: u16, api_bind_address: &str) -> String {
    // Check for environment variable override
    if let Ok(override_endpoint) = std::env::var("K3D_API_ENDPOINT") {
        return override_endpoint;
    }
    
    // Check if we're inside a container
    let in_container = is_running_in_container();
    
    if in_container && api_bind_address == "0.0.0.0" {
        // Inside container with 0.0.0.0 binding, use host.docker.internal
        format!("https://host.docker.internal:{}", api_port)
    } else {
        // Outside container or non-0.0.0.0 binding, use localhost
        format!("https://localhost:{}", api_port)
    }
}
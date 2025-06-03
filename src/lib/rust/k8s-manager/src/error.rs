//! Error types for k8s_manager

use thiserror::Error;

/// Result type alias for k8s_manager operations
pub type Result<T> = std::result::Result<T, K8sManagerError>;

/// Errors that can occur during Kubernetes cluster management
#[derive(Error, Debug)]
pub enum K8sManagerError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Process execution error
    #[error("Process execution error: {0}")]
    ProcessError(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Cluster already exists
    #[error("Cluster '{0}' already exists")]
    ClusterAlreadyExists(String),
    
    /// Cluster not found
    #[error("Cluster '{0}' not found")]
    ClusterNotFound(String),
    
    /// Tool not installed
    #[error("Required tool '{0}' is not installed")]
    ToolNotInstalled(String),
    
    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// DNS resolution error
    #[error("DNS resolution failed: {0}")]
    DnsError(String),
    
    /// Network configuration error
    #[error("Network configuration error: {0}")]
    NetworkError(String),
    
    /// TLS/Certificate error
    #[error("TLS/Certificate error: {0}")]
    TlsError(String),
    
    /// General error
    #[error("General error: {0}")]
    GeneralError(String),
}

impl From<std::io::Error> for K8sManagerError {
    fn from(err: std::io::Error) -> Self {
        K8sManagerError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for K8sManagerError {
    fn from(err: serde_json::Error) -> Self {
        K8sManagerError::ConfigError(err.to_string())
    }
}

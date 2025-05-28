//! Error types for Firestream
//!
//! This module defines custom error types and error handling utilities.

use std::fmt;

/// Custom error type for Firestream operations
#[derive(Debug)]
pub enum FirestreamError {
    /// Configuration-related errors
    ConfigError(String),
    
    /// Service not found
    ServiceNotFound(String),
    
    /// Dependency error
    DependencyError(String),
    
    /// Resource constraint error
    ResourceConstraint(String),
    
    /// Kubernetes API error
    KubernetesError(String),
    
    /// IO error
    IoError(std::io::Error),
    
    /// General error with message
    GeneralError(String),
}

impl fmt::Display for FirestreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FirestreamError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            FirestreamError::ServiceNotFound(service) => write!(f, "Service not found: {}", service),
            FirestreamError::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
            FirestreamError::ResourceConstraint(msg) => write!(f, "Resource constraint: {}", msg),
            FirestreamError::KubernetesError(msg) => write!(f, "Kubernetes error: {}", msg),
            FirestreamError::IoError(err) => write!(f, "IO error: {}", err),
            FirestreamError::GeneralError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for FirestreamError {}

impl From<std::io::Error> for FirestreamError {
    fn from(err: std::io::Error) -> Self {
        FirestreamError::IoError(err)
    }
}

impl From<anyhow::Error> for FirestreamError {
    fn from(err: anyhow::Error) -> Self {
        FirestreamError::GeneralError(err.to_string())
    }
}

/// Result type alias for Firestream operations
pub type Result<T> = std::result::Result<T, FirestreamError>;

/// Convert FirestreamError to appropriate exit code
pub fn error_to_exit_code(error: &FirestreamError) -> i32 {
    match error {
        FirestreamError::ConfigError(_) => 2,
        FirestreamError::ServiceNotFound(_) => 3,
        FirestreamError::DependencyError(_) => 4,
        FirestreamError::ResourceConstraint(_) => 5,
        _ => 1, // General error
    }
}

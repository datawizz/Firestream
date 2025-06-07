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
    IoError(String),
    
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
            FirestreamError::IoError(msg) => write!(f, "IO error: {}", msg),
            FirestreamError::GeneralError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for FirestreamError {}

impl From<std::io::Error> for FirestreamError {
    fn from(err: std::io::Error) -> Self {
        FirestreamError::IoError(err.to_string())
    }
}



impl From<anyhow::Error> for FirestreamError {
    fn from(err: anyhow::Error) -> Self {
        FirestreamError::GeneralError(err.to_string())
    }
}

impl From<serde_json::Error> for FirestreamError {
    fn from(err: serde_json::Error) -> Self {
        FirestreamError::GeneralError(format!("JSON error: {}", err))
    }
}

impl From<toml::ser::Error> for FirestreamError {
    fn from(err: toml::ser::Error) -> Self {
        FirestreamError::GeneralError(format!("TOML serialization error: {}", err))
    }
}

impl From<toml::de::Error> for FirestreamError {
    fn from(err: toml::de::Error) -> Self {
        FirestreamError::GeneralError(format!("TOML deserialization error: {}", err))
    }
}

impl From<tera::Error> for FirestreamError {
    fn from(err: tera::Error) -> Self {
        FirestreamError::GeneralError(format!("Template error: {}", err))
    }
}

impl From<dialoguer::Error> for FirestreamError {
    fn from(err: dialoguer::Error) -> Self {
        FirestreamError::GeneralError(format!("Dialog error: {}", err))
    }
}

impl From<serde_yaml::Error> for FirestreamError {
    fn from(err: serde_yaml::Error) -> Self {
        FirestreamError::GeneralError(format!("YAML error: {}", err))
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

//! Error types for docker-manager

use thiserror::Error;

/// Result type alias for docker-manager operations
pub type Result<T> = std::result::Result<T, DockerManagerError>;

/// Errors that can occur during Docker operations
#[derive(Error, Debug)]
pub enum DockerManagerError {
    /// Docker daemon not accessible
    #[error("Docker daemon is not accessible: {0}")]
    DockerNotAccessible(String),
    
    /// Container not found
    #[error("Container '{0}' not found")]
    ContainerNotFound(String),
    
    /// Image not found
    #[error("Image '{0}' not found")]
    ImageNotFound(String),
    
    /// Volume not found
    #[error("Volume '{0}' not found")]
    VolumeNotFound(String),
    
    /// Network not found
    #[error("Network '{0}' not found")]
    NetworkNotFound(String),
    
    /// Container already exists
    #[error("Container '{0}' already exists")]
    ContainerAlreadyExists(String),
    
    /// Invalid container state
    #[error("Invalid container state: {0}")]
    InvalidContainerState(String),
    
    /// Build error
    #[error("Build error: {0}")]
    BuildError(String),
    
    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Docker API error
    #[error("Docker API error: {0}")]
    DockerApiError(String),
    
    /// General error
    #[error("General error: {0}")]
    GeneralError(String),
}

impl From<bollard::errors::Error> for DockerManagerError {
    fn from(err: bollard::errors::Error) -> Self {
        DockerManagerError::DockerApiError(err.to_string())
    }
}

impl From<std::io::Error> for DockerManagerError {
    fn from(err: std::io::Error) -> Self {
        DockerManagerError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for DockerManagerError {
    fn from(err: serde_json::Error) -> Self {
        DockerManagerError::SerializationError(err.to_string())
    }
}
//! Error types for nix-container-builder

use std::path::PathBuf;
use std::time::Duration;

/// Custom error type for nix-container-builder operations
#[derive(Debug, thiserror::Error)]
pub enum NixContainerError {
    /// Nix is not installed or not found in PATH
    #[error("Nix not installed or not in PATH")]
    NixNotInstalled,

    /// Docker daemon is not accessible
    #[error("Docker not accessible: {0}")]
    DockerNotAccessible(String),

    /// Flake.nix file not found at the expected path
    #[error("Flake not found at {}", path.display())]
    FlakeNotFound { path: PathBuf },

    /// Container not found in the containers directory
    #[error("Container '{name}' not found in {}", containers_dir.display())]
    ContainerNotFound {
        name: String,
        containers_dir: PathBuf,
    },

    /// Build process failed
    #[error("Build failed for '{container}': {message}")]
    BuildFailed { container: String, message: String },

    /// Docker load command failed
    #[error("Docker load failed: {0}")]
    DockerLoadFailed(String),

    /// Platform is not supported for the requested operation
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    /// Operation timed out
    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    /// Process execution error
    #[error("Process error: {0}")]
    ProcessError(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Generic error for other cases
    #[error("{0}")]
    Other(String),
}

/// Result type alias using NixContainerError
pub type Result<T> = std::result::Result<T, NixContainerError>;

impl NixContainerError {
    /// Create a build failed error
    pub fn build_failed(container: impl Into<String>, message: impl Into<String>) -> Self {
        Self::BuildFailed {
            container: container.into(),
            message: message.into(),
        }
    }

    /// Create a container not found error
    pub fn container_not_found(name: impl Into<String>, containers_dir: PathBuf) -> Self {
        Self::ContainerNotFound {
            name: name.into(),
            containers_dir,
        }
    }

    /// Create an other error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

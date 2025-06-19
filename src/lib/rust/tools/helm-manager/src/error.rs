use thiserror::Error;

/// Result type alias for helm-manager operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for helm-manager
#[derive(Error, Debug)]
pub enum Error {
    /// Helm command not found
    #[error("Helm command not found. Please ensure helm is installed and in PATH")]
    HelmNotFound,

    /// Kubectl command not found
    #[error("Kubectl command not found. Please ensure kubectl is installed and in PATH")]
    KubectlNotFound,

    /// Chart not found
    #[error("Chart '{0}' not found")]
    ChartNotFound(String),

    /// Release not found
    #[error("Release '{0}' not found")]
    ReleaseNotFound(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Invalid values
    #[error("Invalid values: {0}")]
    InvalidValues(String),

    /// Helm command failed
    #[error("Helm command failed: {0}")]
    HelmCommandFailed(String),

    /// Kubectl command failed
    #[error("Kubectl command failed: {0}")]
    KubectlCommandFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Environment file error
    #[error("Failed to load environment file: {0}")]
    EnvFile(String),

    /// Template extraction error
    #[error("Failed to extract chart: {0}")]
    ChartExtraction(String),

    /// Deployment error
    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),

    /// Rollback error
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    /// Other errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}
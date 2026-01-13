//! Test runner module
//!
//! Executes tests in various environments (Docker, Kubernetes) and
//! with various tools (Goss, Trivy, Grype).

pub mod docker;
pub mod goss;
pub mod grype;
pub mod kubernetes;
pub mod trivy;

pub use docker::DockerRunner;
pub use goss::GossRunner;
pub use grype::GrypeRunner;
pub use kubernetes::KubernetesRunner;
pub use trivy::TrivyRunner;

/// Error type for runner operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Docker error: {0}")]
    Docker(String),

    #[error("Kubernetes error: {0}")]
    Kubernetes(String),

    #[error("Goss test failed: {0}")]
    GossFailed(String),

    #[error("Security scan failed: {0}")]
    SecurityScanFailed(String),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Test execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Success status
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Output/logs
    pub output: String,
    /// Error message (if failed)
    pub error: Option<String>,
}

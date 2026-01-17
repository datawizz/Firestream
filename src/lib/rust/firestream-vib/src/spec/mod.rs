//! Test specification data structures
//!
//! Defines the schema for container test specifications, including
//! Goss tests, security requirements, and runtime configuration.

pub mod container;
pub mod goss;
pub mod security;

pub use container::ContainerSpec;
pub use goss::GossSpec;
pub use security::SecuritySpec;

/// Error type for spec operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid spec format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

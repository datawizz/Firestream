//! Shared Firestream container metadata library.
//!
//! Centralizes the schema definitions (CycloneDX 1.5, SPDX 2.3, Nix closure JSON,
//! firestream metadata.json) and the runtime reader/validator that consume them.
//!
//! This crate exists to ensure that the *generator* (build-time, in `firestream-vib`),
//! the *runtime servers* (`firestream-healthd`), and the *fleet manifest* code path
//! all share a single source of truth for the metadata schemas. Std deps only —
//! no bollard, kube, tera, axum, or tokio.
//!
//! # Modules
//!
//! - [`reader`] — Reads and parses the metadata files baked into containers at
//!   `/opt/firestream/`.
//! - [`validator`] — Validates the structure and content of those files.
//! - [`spec`] — Schema types for CycloneDX 1.5, SPDX 2.3, the firestream
//!   `closure.json`, and the generator input config (`MetadataConfig`).

pub mod reader;
pub mod spec;
pub mod validator;

pub use reader::MetadataReader;
pub use validator::{Check, MetadataValidator, ValidationResult};

/// Error type for metadata operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Metadata file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to parse metadata: {0}")]
    ParseError(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

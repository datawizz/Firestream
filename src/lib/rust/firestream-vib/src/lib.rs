//! Firestream VIB (Verification and Integration for Bitnami-style containers)
//!
//! This library provides a comprehensive testing harness for containerized applications
//! built with Nix. It integrates Goss for structural testing, Trivy and Grype for
//! security scanning, and supports both Docker and Kubernetes execution environments.
//!
//! # Architecture
//!
//! - `nix`: Nix command invocation and metadata parsing
//! - `spec`: Data structures for test specifications
//! - `generator`: Goss YAML generation from templates
//! - `runner`: Test execution (Docker, K8s, Goss, Trivy, Grype)
//! - `report`: Output formatting (JSON, JUnit, SARIF)
//! - `cache`: Result caching by Nix hash
//! - `metadata`: Container metadata reading and validation
//! - `merge`: SBOM merging for fleet manifests

pub mod cache;
pub mod generator;
pub mod merge;
pub mod metadata;
pub mod nix;
pub mod report;
pub mod runner;
pub mod source_archive;
pub mod spec;

pub use cache::NixHashCache;
pub use merge::{MergeError, OutputFormat, SbomMerger};
pub use metadata::{MetadataReader, MetadataValidator};
pub use nix::{ClosureGraph, MetadataConfig};
pub use source_archive::{SourceArchiver, SourceIndex, SourceMap, SourceType};
pub use spec::{ContainerSpec, GossSpec, SecuritySpec};

/// Result type alias for firestream-vib operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for firestream-vib operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Nix error: {0}")]
    Nix(#[from] nix::Error),

    #[error("Spec error: {0}")]
    Spec(#[from] spec::Error),

    #[error("Generator error: {0}")]
    Generator(#[from] generator::Error),

    #[error("Runner error: {0}")]
    Runner(#[from] runner::Error),

    #[error("Report error: {0}")]
    Report(#[from] report::Error),

    #[error("Cache error: {0}")]
    Cache(#[from] cache::Error),

    #[error("Metadata error: {0}")]
    Metadata(#[from] metadata::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

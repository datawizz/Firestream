//! Nix integration module
//!
//! Provides functionality for interacting with Nix to build containers,
//! extract metadata, and generate SBOMs.

pub mod closure_graph;
pub mod derivation;
pub mod flake;
pub mod metadata;
pub mod sbom;

pub use closure_graph::{ClosureGraph, MetadataConfig};
pub use derivation::Derivation;
pub use flake::FlakeInfo;
pub use metadata::NixMetadata;
pub use sbom::Sbom;

/// Error type for Nix operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Nix command failed: {0}")]
    CommandFailed(String),

    #[error("Failed to parse Nix output: {0}")]
    ParseError(String),

    #[error("Derivation not found: {0}")]
    DerivationNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

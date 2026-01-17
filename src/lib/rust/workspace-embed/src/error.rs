//! Error types for workspace-embed operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during workspace embedding operations.
#[derive(Error, Debug)]
pub enum WorkspaceEmbedError {
    /// The specified source path does not exist.
    #[error("source path not found: {path}")]
    SourceNotFound { path: PathBuf },

    /// Failed to parse an ignore file (gitignore, dockerignore, etc.).
    #[error("failed to parse ignore file at {path}: {reason}")]
    IgnoreParseError { path: PathBuf, reason: String },

    /// Git repository not found at the specified path.
    #[error("git repository not found at {path}")]
    GitNotFound { path: PathBuf },

    /// Error accessing a git object.
    #[error("git object error for oid {oid}")]
    GitObjectError { oid: String },

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// An error occurred during extraction.
    #[error("extraction error: {reason}")]
    ExtractionError { reason: String },
}

/// A specialized Result type for workspace-embed operations.
pub type Result<T> = std::result::Result<T, WorkspaceEmbedError>;

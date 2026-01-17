//! Workspace embedding with gitignore support for Nix builds.
//!
//! This crate provides utilities for embedding workspace files into Rust binaries
//! with support for .gitignore and .dockerignore patterns. It is designed to work
//! seamlessly with Nix builds where file filtering is important for reproducibility.
//!
//! # Overview
//!
//! The main workflow is:
//! 1. Configure which files/directories to include via [`EmbedConfig`]
//! 2. Collect files respecting ignore patterns
//! 3. Extract files to output directory or embed at compile time
//!
//! # Example
//!
//! ```rust,ignore
//! use workspace_embed::{EmbedConfig, GitConfig};
//!
//! let config = EmbedConfig::new("./src", "./embedded")
//!     .with_gitignore(true)
//!     .with_git_config(GitConfig::Minimal)
//!     .with_include_dirs(vec!["charts".into(), "templates".into()]);
//! ```

pub mod collector;
pub mod config;
pub mod error;
pub mod extractor;
pub mod git;
pub mod ignore;

// Re-export key types for convenient access
pub use collector::{EmbedBuilder, EmbedResult};
pub use config::{EmbedConfig, GitConfig};
pub use error::{Result, WorkspaceEmbedError};
pub use extractor::ExtractedWorkspace;
pub use git::{MinimalGitCreator, MinimalGitInfo};
pub use ignore::IgnoreFilter;

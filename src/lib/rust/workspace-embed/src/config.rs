//! Configuration types for workspace embedding.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for git-related embedding behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum GitConfig {
    /// Do not include any git information.
    #[default]
    None,
    /// Include minimal git information (e.g., current commit hash).
    Minimal,
}

/// Configuration for the workspace embedding process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedConfig {
    /// Root directory of the source workspace.
    pub source_root: PathBuf,

    /// Output directory for embedded files.
    pub output_dir: PathBuf,

    /// Specific directories to include (relative to source_root).
    /// If empty, all directories are considered.
    pub include_dirs: Vec<PathBuf>,

    /// Specific files to include (relative to source_root).
    /// If empty, all files are considered (subject to ignore rules).
    pub include_files: Vec<PathBuf>,

    /// Whether to respect .gitignore files when collecting files.
    pub respect_gitignore: bool,

    /// Whether to respect .dockerignore files when collecting files.
    pub respect_dockerignore: bool,

    /// Git configuration for the embedding process.
    pub git_config: GitConfig,

    /// Additional patterns to exclude (glob patterns).
    pub exclude_patterns: Vec<String>,
}

impl Default for EmbedConfig {
    fn default() -> Self {
        Self {
            source_root: PathBuf::from("."),
            output_dir: PathBuf::from("embedded"),
            include_dirs: Vec::new(),
            include_files: Vec::new(),
            respect_gitignore: true,
            respect_dockerignore: false,
            git_config: GitConfig::default(),
            exclude_patterns: Vec::new(),
        }
    }
}

impl EmbedConfig {
    /// Create a new EmbedConfig with the given source root and output directory.
    pub fn new(source_root: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            source_root: source_root.into(),
            output_dir: output_dir.into(),
            ..Default::default()
        }
    }

    /// Set whether to respect gitignore files.
    pub fn with_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    /// Set whether to respect dockerignore files.
    pub fn with_dockerignore(mut self, respect: bool) -> Self {
        self.respect_dockerignore = respect;
        self
    }

    /// Set the git configuration.
    pub fn with_git_config(mut self, config: GitConfig) -> Self {
        self.git_config = config;
        self
    }

    /// Add directories to include.
    pub fn with_include_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.include_dirs = dirs;
        self
    }

    /// Add files to include.
    pub fn with_include_files(mut self, files: Vec<PathBuf>) -> Self {
        self.include_files = files;
        self
    }

    /// Add patterns to exclude.
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }
}

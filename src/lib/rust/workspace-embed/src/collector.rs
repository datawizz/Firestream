//! File collection and embedding for build.rs.
//!
//! This module provides a builder pattern for configuring and executing file embedding
//! at compile time. It walks directory trees, respects ignore patterns, and copies
//! files to an output directory for inclusion in the final binary.
//!
//! # Overview
//!
//! The [`EmbedBuilder`] provides a fluent API for:
//! - Specifying source directories and files to include
//! - Configuring gitignore/dockerignore respect
//! - Setting up minimal .git creation for Nix flake compatibility
//! - Executing the embedding and generating cargo rerun-if-changed directives
//!
//! # Example (in build.rs)
//!
//! ```rust,ignore
//! use workspace_embed::collector::EmbedBuilder;
//! use std::path::PathBuf;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
//!     let repo_root = PathBuf::from(&manifest_dir).ancestors().nth(4).unwrap();
//!
//!     let result = EmbedBuilder::new()
//!         .source_root(&repo_root)
//!         .output_dir(PathBuf::from(&manifest_dir).join("embedded"))
//!         .include_dir("src/charts")
//!         .include_dir("src/containers")
//!         .include_file("flake.nix")
//!         .respect_gitignore(true)
//!         .git_minimal()
//!         .exclude("*.pyc")
//!         .build()?;
//!
//!     println!("cargo:warning=Embedded {} files", result.file_count);
//!     Ok(())
//! }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use tracing::{debug, info, trace, warn};
use walkdir::WalkDir;

use crate::config::GitConfig;
use crate::error::{Result, WorkspaceEmbedError};
use crate::git::MinimalGitCreator;
use crate::ignore::IgnoreFilter;

/// Result of a successful embedding operation.
#[derive(Debug, Clone)]
pub struct EmbedResult {
    /// Number of files embedded.
    pub file_count: usize,
    /// Total size of all embedded files in bytes.
    pub total_size: u64,
    /// Path to the output directory containing embedded files.
    pub output_dir: PathBuf,
    /// List of embedded files (relative paths from output_dir).
    pub files: Vec<PathBuf>,
}

/// Builder for configuring and executing file embedding.
///
/// This struct implements the builder pattern for setting up file embedding
/// at compile time. It collects configuration options and then executes
/// the embedding when [`build()`](EmbedBuilder::build) is called.
#[derive(Debug, Clone)]
pub struct EmbedBuilder {
    /// Root directory of the source workspace.
    source_root: Option<PathBuf>,
    /// Output directory for embedded files.
    output_dir: Option<PathBuf>,
    /// Directories to include (relative to source_root).
    include_dirs: Vec<PathBuf>,
    /// Individual files to include (relative to source_root).
    include_files: Vec<PathBuf>,
    /// Whether to respect .gitignore patterns.
    respect_gitignore: bool,
    /// Whether to respect .dockerignore patterns.
    respect_dockerignore: bool,
    /// Git configuration for the embedded output.
    git_config: GitConfig,
    /// Additional exclude patterns (glob format).
    exclude_patterns: Vec<String>,
}

impl Default for EmbedBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbedBuilder {
    /// Create a new EmbedBuilder with default settings.
    ///
    /// Default settings:
    /// - No source_root (must be set)
    /// - No output_dir (must be set)
    /// - No include_dirs or include_files
    /// - respect_gitignore: true
    /// - respect_dockerignore: false
    /// - git_config: None
    /// - No exclude patterns
    pub fn new() -> Self {
        Self {
            source_root: None,
            output_dir: None,
            include_dirs: Vec::new(),
            include_files: Vec::new(),
            respect_gitignore: true,
            respect_dockerignore: false,
            git_config: GitConfig::None,
            exclude_patterns: Vec::new(),
        }
    }

    /// Set the source root directory.
    ///
    /// All include paths will be resolved relative to this directory.
    /// The source root is also used to find .gitignore and .dockerignore files.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the source root directory
    pub fn source_root(mut self, path: impl AsRef<Path>) -> Self {
        self.source_root = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the output directory for embedded files.
    ///
    /// This directory will be cleared and recreated during the build process.
    /// All embedded files will be copied here preserving their relative structure.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output directory
    pub fn output_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.output_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Add a directory to embed.
    ///
    /// The directory and all its contents (respecting ignore patterns) will be
    /// copied to the output directory, preserving the directory structure.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory (relative to source_root)
    pub fn include_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.include_dirs.push(path.as_ref().to_path_buf());
        self
    }

    /// Add a single file to embed.
    ///
    /// The file will be copied to the output directory. Ignore patterns
    /// are not applied to explicitly included files.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file (relative to source_root)
    pub fn include_file(mut self, path: impl AsRef<Path>) -> Self {
        self.include_files.push(path.as_ref().to_path_buf());
        self
    }

    /// Set whether to respect .gitignore patterns.
    ///
    /// When enabled, files matching patterns in .gitignore files will be
    /// excluded from the embedded output.
    ///
    /// # Arguments
    ///
    /// * `respect` - true to respect .gitignore, false to ignore it
    pub fn respect_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    /// Set whether to respect .dockerignore patterns.
    ///
    /// When enabled, files matching patterns in .dockerignore files will be
    /// excluded from the embedded output.
    ///
    /// # Arguments
    ///
    /// * `respect` - true to respect .dockerignore, false to ignore it
    pub fn respect_dockerignore(mut self, respect: bool) -> Self {
        self.respect_dockerignore = respect;
        self
    }

    /// Enable minimal .git embedding.
    ///
    /// This creates a minimal .git directory in the output that contains
    /// only the essential files needed for Nix flake resolution:
    /// - HEAD
    /// - config
    /// - refs/heads/{branch}
    /// - Commit and tree objects for HEAD
    pub fn git_minimal(mut self) -> Self {
        self.git_config = GitConfig::Minimal;
        self
    }

    /// Disable .git embedding.
    ///
    /// No .git directory will be created in the output.
    pub fn git_none(mut self) -> Self {
        self.git_config = GitConfig::None;
        self
    }

    /// Add an exclude pattern.
    ///
    /// Files matching this glob pattern will be excluded from embedding.
    /// Multiple patterns can be added by calling this method multiple times.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Glob pattern to exclude (e.g., "*.pyc", "**/__pycache__/**")
    pub fn exclude(mut self, pattern: &str) -> Self {
        self.exclude_patterns.push(pattern.to_string());
        self
    }

    /// Execute the embedding process.
    ///
    /// This method:
    /// 1. Validates that source_root and output_dir are set
    /// 2. Cleans and recreates the output directory
    /// 3. Creates an IgnoreFilter from the source root
    /// 4. Walks each include_dir, filtering with the IgnoreFilter, and copies files
    /// 5. Copies each include_file directly
    /// 6. If git_config is Minimal, creates a minimal .git directory
    /// 7. Prints cargo:rerun-if-changed directives
    /// 8. Returns an EmbedResult with statistics
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - source_root or output_dir is not set
    /// - source_root does not exist
    /// - Any I/O operation fails
    /// - Ignore pattern parsing fails
    pub fn build(self) -> Result<EmbedResult> {
        // Validate required fields
        let source_root = self.source_root.ok_or_else(|| {
            WorkspaceEmbedError::ExtractionError {
                reason: "source_root must be set".to_string(),
            }
        })?;

        let output_dir = self.output_dir.ok_or_else(|| {
            WorkspaceEmbedError::ExtractionError {
                reason: "output_dir must be set".to_string(),
            }
        })?;

        // Canonicalize source root
        let source_root = source_root.canonicalize().map_err(|_| {
            WorkspaceEmbedError::SourceNotFound {
                path: source_root.clone(),
            }
        })?;

        info!(
            source_root = %source_root.display(),
            output_dir = %output_dir.display(),
            include_dirs = ?self.include_dirs,
            include_files = ?self.include_files,
            "Starting file embedding"
        );

        // Clean and create output directory
        if output_dir.exists() {
            debug!(output_dir = %output_dir.display(), "Removing existing output directory");
            fs::remove_dir_all(&output_dir)?;
        }
        fs::create_dir_all(&output_dir)?;

        // Create ignore filter
        let mut filter = IgnoreFilter::from_root(
            &source_root,
            self.respect_gitignore,
            self.respect_dockerignore,
        )?;

        // Add custom exclude patterns
        if !self.exclude_patterns.is_empty() {
            filter.add_excludes(self.exclude_patterns.iter().map(|s| s.as_str()))?;
        }

        let mut file_count = 0usize;
        let mut total_size = 0u64;
        let mut files = Vec::new();

        // Process include directories
        for include_dir in &self.include_dirs {
            let full_dir = source_root.join(include_dir);

            if !full_dir.exists() {
                warn!(
                    dir = %full_dir.display(),
                    "Include directory does not exist, skipping"
                );
                continue;
            }

            // Print rerun-if-changed for the directory
            println!("cargo:rerun-if-changed={}", full_dir.display());

            debug!(dir = %full_dir.display(), "Walking directory");

            for entry in WalkDir::new(&full_dir)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    // Skip directories that should be ignored
                    let is_dir = e.file_type().is_dir();
                    let path = e.path();

                    // Get path relative to source_root for filtering
                    let relative = path
                        .strip_prefix(&source_root)
                        .unwrap_or(path);

                    filter.should_include(relative, is_dir)
                })
            {
                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        warn!(error = %err, "Error walking directory entry");
                        continue;
                    }
                };

                // Skip directories themselves (we only copy files)
                if entry.file_type().is_dir() {
                    continue;
                }

                let src_path = entry.path();

                // Get relative path from source root
                let relative_path = src_path
                    .strip_prefix(&source_root)
                    .map_err(|_| WorkspaceEmbedError::ExtractionError {
                        reason: format!(
                            "Failed to get relative path for {}",
                            src_path.display()
                        ),
                    })?;

                // Check if file should be included
                if !filter.should_include(relative_path, false) {
                    trace!(path = %relative_path.display(), "Skipping ignored file");
                    continue;
                }

                // Copy the file
                let dest_path = output_dir.join(relative_path);
                let (size, copied_relative) = copy_file(src_path, &dest_path, relative_path)?;

                file_count += 1;
                total_size += size;
                files.push(copied_relative);

                trace!(
                    src = %src_path.display(),
                    dest = %dest_path.display(),
                    size,
                    "Copied file"
                );
            }
        }

        // Process individual include files
        for include_file in &self.include_files {
            let full_path = source_root.join(include_file);

            if !full_path.exists() {
                warn!(
                    file = %full_path.display(),
                    "Include file does not exist, skipping"
                );
                continue;
            }

            if !full_path.is_file() {
                warn!(
                    path = %full_path.display(),
                    "Include path is not a file, skipping"
                );
                continue;
            }

            // Print rerun-if-changed for the file
            println!("cargo:rerun-if-changed={}", full_path.display());

            // Copy the file (ignore patterns are not applied to explicit includes)
            let dest_path = output_dir.join(include_file);
            let (size, copied_relative) = copy_file(&full_path, &dest_path, include_file)?;

            file_count += 1;
            total_size += size;
            files.push(copied_relative);

            debug!(
                src = %full_path.display(),
                dest = %dest_path.display(),
                size,
                "Copied explicit include file"
            );
        }

        // Create minimal .git if configured
        if self.git_config == GitConfig::Minimal {
            match MinimalGitCreator::from_repo(&source_root) {
                Ok(creator) => {
                    match creator.create_minimal(&output_dir) {
                        Ok(git_info) => {
                            info!(
                                branch = %git_info.branch,
                                commit = %git_info.commit,
                                git_dir = %git_info.git_dir.display(),
                                "Created minimal .git directory"
                            );
                            // Count .git files
                            for entry in WalkDir::new(&git_info.git_dir)
                                .into_iter()
                                .filter_map(|e| e.ok())
                            {
                                if entry.file_type().is_file() {
                                    if let Ok(metadata) = entry.metadata() {
                                        total_size += metadata.len();
                                        file_count += 1;
                                        if let Ok(rel) = entry.path().strip_prefix(&output_dir) {
                                            files.push(rel.to_path_buf());
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            warn!(error = %err, "Failed to create minimal .git, continuing without it");
                        }
                    }
                }
                Err(err) => {
                    warn!(error = %err, "Failed to find source .git, continuing without minimal .git");
                }
            }

            // Add rerun-if-changed for .git/HEAD (tracks branch/commit changes)
            let git_head = source_root.join(".git/HEAD");
            if git_head.exists() {
                println!("cargo:rerun-if-changed={}", git_head.display());
            }
        }

        info!(
            file_count,
            total_size,
            output_dir = %output_dir.display(),
            "Embedding complete"
        );

        Ok(EmbedResult {
            file_count,
            total_size,
            output_dir,
            files,
        })
    }
}

/// Copy a file to the destination, creating parent directories as needed.
///
/// # Arguments
///
/// * `src` - Source file path
/// * `dest` - Destination file path
/// * `relative` - Relative path for tracking (returned in result)
///
/// # Returns
///
/// A tuple of (bytes_copied, relative_path).
fn copy_file(src: &Path, dest: &Path, relative: &Path) -> Result<(u64, PathBuf)> {
    // Create parent directories
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy the file
    let bytes = fs::copy(src, dest)?;

    Ok((bytes, relative.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_workspace() -> TempDir {
        let temp = TempDir::new().expect("Failed to create temp dir");

        // Create directory structure
        let src_dir = temp.path().join("src");
        let charts_dir = temp.path().join("src/charts");
        let containers_dir = temp.path().join("src/containers");

        fs::create_dir_all(&charts_dir).unwrap();
        fs::create_dir_all(&containers_dir).unwrap();

        // Create some files
        fs::write(temp.path().join("flake.nix"), "{ inputs = {}; }").unwrap();
        fs::write(temp.path().join("flake.lock"), "{}").unwrap();
        fs::write(charts_dir.join("Chart.yaml"), "name: test").unwrap();
        fs::write(containers_dir.join("Dockerfile"), "FROM alpine").unwrap();

        // Create a file that should be ignored
        fs::write(src_dir.join("test.pyc"), "bytecode").unwrap();

        // Create a .gitignore
        fs::write(temp.path().join(".gitignore"), "*.pyc\n*.log\n").unwrap();

        temp
    }

    #[test]
    fn test_embed_builder_defaults() {
        let builder = EmbedBuilder::new();

        assert!(builder.source_root.is_none());
        assert!(builder.output_dir.is_none());
        assert!(builder.include_dirs.is_empty());
        assert!(builder.include_files.is_empty());
        assert!(builder.respect_gitignore);
        assert!(!builder.respect_dockerignore);
        assert_eq!(builder.git_config, GitConfig::None);
        assert!(builder.exclude_patterns.is_empty());
    }

    #[test]
    fn test_embed_builder_fluent_api() {
        let builder = EmbedBuilder::new()
            .source_root("/src")
            .output_dir("/out")
            .include_dir("charts")
            .include_dir("containers")
            .include_file("flake.nix")
            .respect_gitignore(true)
            .respect_dockerignore(true)
            .git_minimal()
            .exclude("*.pyc")
            .exclude("__pycache__");

        assert_eq!(builder.source_root, Some(PathBuf::from("/src")));
        assert_eq!(builder.output_dir, Some(PathBuf::from("/out")));
        assert_eq!(builder.include_dirs.len(), 2);
        assert_eq!(builder.include_files.len(), 1);
        assert!(builder.respect_gitignore);
        assert!(builder.respect_dockerignore);
        assert_eq!(builder.git_config, GitConfig::Minimal);
        assert_eq!(builder.exclude_patterns.len(), 2);
    }

    #[test]
    fn test_embed_builder_git_none() {
        let builder = EmbedBuilder::new()
            .git_minimal()
            .git_none();

        assert_eq!(builder.git_config, GitConfig::None);
    }

    #[test]
    fn test_build_requires_source_root() {
        let temp = TempDir::new().unwrap();

        let result = EmbedBuilder::new()
            .output_dir(temp.path().join("output"))
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, WorkspaceEmbedError::ExtractionError { .. }));
    }

    #[test]
    fn test_build_requires_output_dir() {
        let temp = create_test_workspace();

        let result = EmbedBuilder::new()
            .source_root(temp.path())
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, WorkspaceEmbedError::ExtractionError { .. }));
    }

    #[test]
    fn test_build_with_include_files() {
        let temp = create_test_workspace();
        let output = TempDir::new().unwrap();

        let result = EmbedBuilder::new()
            .source_root(temp.path())
            .output_dir(output.path().join("embedded"))
            .include_file("flake.nix")
            .include_file("flake.lock")
            .build()
            .expect("Build should succeed");

        assert_eq!(result.file_count, 2);
        assert!(result.total_size > 0);
        assert!(result.files.contains(&PathBuf::from("flake.nix")));
        assert!(result.files.contains(&PathBuf::from("flake.lock")));

        // Verify files exist in output
        let embedded_dir = output.path().join("embedded");
        assert!(embedded_dir.join("flake.nix").exists());
        assert!(embedded_dir.join("flake.lock").exists());
    }

    #[test]
    fn test_build_with_include_dirs() {
        let temp = create_test_workspace();
        let output = TempDir::new().unwrap();

        let result = EmbedBuilder::new()
            .source_root(temp.path())
            .output_dir(output.path().join("embedded"))
            .include_dir("src/charts")
            .respect_gitignore(false)
            .build()
            .expect("Build should succeed");

        assert!(result.file_count >= 1);
        assert!(result.total_size > 0);

        // Verify files exist in output
        let embedded_dir = output.path().join("embedded");
        assert!(embedded_dir.join("src/charts/Chart.yaml").exists());
    }

    #[test]
    fn test_build_respects_gitignore() {
        let temp = create_test_workspace();
        let output = TempDir::new().unwrap();

        // First, create a .pyc file in charts dir
        let charts_dir = temp.path().join("src/charts");
        fs::write(charts_dir.join("cache.pyc"), "bytecode").unwrap();

        let _result = EmbedBuilder::new()
            .source_root(temp.path())
            .output_dir(output.path().join("embedded"))
            .include_dir("src/charts")
            .respect_gitignore(true)
            .build()
            .expect("Build should succeed");

        // The .pyc file should be ignored
        let embedded_dir = output.path().join("embedded");
        assert!(!embedded_dir.join("src/charts/cache.pyc").exists());

        // But Chart.yaml should exist
        assert!(embedded_dir.join("src/charts/Chart.yaml").exists());
    }

    #[test]
    fn test_build_with_custom_excludes() {
        let temp = create_test_workspace();
        let output = TempDir::new().unwrap();

        // Create a .log file
        let charts_dir = temp.path().join("src/charts");
        fs::write(charts_dir.join("debug.log"), "log content").unwrap();
        fs::write(charts_dir.join("notes.txt"), "notes").unwrap();

        let _result = EmbedBuilder::new()
            .source_root(temp.path())
            .output_dir(output.path().join("embedded"))
            .include_dir("src/charts")
            .respect_gitignore(false)
            .exclude("*.log")
            .exclude("*.txt")
            .build()
            .expect("Build should succeed");

        let embedded_dir = output.path().join("embedded");

        // .log and .txt files should be excluded
        assert!(!embedded_dir.join("src/charts/debug.log").exists());
        assert!(!embedded_dir.join("src/charts/notes.txt").exists());

        // But Chart.yaml should exist
        assert!(embedded_dir.join("src/charts/Chart.yaml").exists());
    }

    #[test]
    fn test_build_cleans_output_dir() {
        let temp = create_test_workspace();
        let output = TempDir::new().unwrap();

        let embedded_dir = output.path().join("embedded");

        // Create existing content in output dir
        fs::create_dir_all(&embedded_dir).unwrap();
        fs::write(embedded_dir.join("old_file.txt"), "old content").unwrap();

        // Run build
        let _result = EmbedBuilder::new()
            .source_root(temp.path())
            .output_dir(&embedded_dir)
            .include_file("flake.nix")
            .build()
            .expect("Build should succeed");

        // Old file should be gone
        assert!(!embedded_dir.join("old_file.txt").exists());

        // New file should exist
        assert!(embedded_dir.join("flake.nix").exists());
    }

    #[test]
    fn test_build_handles_missing_include_dir() {
        let temp = create_test_workspace();
        let output = TempDir::new().unwrap();

        // Include a directory that doesn't exist
        let result = EmbedBuilder::new()
            .source_root(temp.path())
            .output_dir(output.path().join("embedded"))
            .include_dir("nonexistent/dir")
            .include_file("flake.nix")
            .build()
            .expect("Build should succeed even with missing dirs");

        // Should still have the explicit file
        assert_eq!(result.file_count, 1);
        assert!(result.files.contains(&PathBuf::from("flake.nix")));
    }

    #[test]
    fn test_build_handles_missing_include_file() {
        let temp = create_test_workspace();
        let output = TempDir::new().unwrap();

        // Include a file that doesn't exist
        let result = EmbedBuilder::new()
            .source_root(temp.path())
            .output_dir(output.path().join("embedded"))
            .include_file("nonexistent.txt")
            .include_file("flake.nix")
            .build()
            .expect("Build should succeed even with missing files");

        // Should still have the existing file
        assert_eq!(result.file_count, 1);
        assert!(result.files.contains(&PathBuf::from("flake.nix")));
    }

    #[test]
    fn test_embed_result_fields() {
        let result = EmbedResult {
            file_count: 10,
            total_size: 1024,
            output_dir: PathBuf::from("/out"),
            files: vec![PathBuf::from("a.txt"), PathBuf::from("b.txt")],
        };

        assert_eq!(result.file_count, 10);
        assert_eq!(result.total_size, 1024);
        assert_eq!(result.output_dir, PathBuf::from("/out"));
        assert_eq!(result.files.len(), 2);
    }
}

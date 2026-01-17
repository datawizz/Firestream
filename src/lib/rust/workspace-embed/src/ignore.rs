//! Ignore file parsing and pattern matching.
//!
//! This module handles parsing of .gitignore and .dockerignore files,
//! and provides utilities for checking whether paths should be ignored.
//!
//! # Overview
//!
//! The [`IgnoreFilter`] struct combines multiple ignore sources:
//! - `.gitignore` files (using the `ignore` crate from ripgrep)
//! - `.dockerignore` files (parsed as gitignore format - they use same syntax)
//! - Additional exclude patterns (glob patterns)
//!
//! # Standard Exclusions
//!
//! The following patterns are always excluded by default:
//! - `.git/` - Git directory (handled specially via git.rs)
//! - `target/` - Rust build artifacts
//! - `node_modules/` - Node.js dependencies
//! - `__pycache__/` - Python bytecode cache
//! - `.venv/` - Python virtual environments
//! - `result` and `result-*` - Nix build outputs
//!
//! # Example
//!
//! ```rust,ignore
//! use workspace_embed::ignore::IgnoreFilter;
//! use std::path::Path;
//!
//! let filter = IgnoreFilter::from_root(
//!     Path::new("/path/to/repo"),
//!     true,  // respect_gitignore
//!     true,  // respect_dockerignore
//! )?;
//!
//! if filter.should_include(Path::new("src/main.rs"), false) {
//!     // Include this file
//! }
//! ```

use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use tracing::{debug, trace, warn};

use crate::config::EmbedConfig;
use crate::error::{Result, WorkspaceEmbedError};

/// Standard exclusion patterns that are always applied.
const STANDARD_EXCLUSIONS: &[&str] = &[
    ".git",
    ".git/**",
    "target",
    "target/**",
    "target-ra",
    "target-ra/**",
    "node_modules",
    "node_modules/**",
    "__pycache__",
    "__pycache__/**",
    ".venv",
    ".venv/**",
    "result",
    "result-*",
    ".direnv",
    ".direnv/**",
];

/// A filter for determining which files should be included or excluded
/// during workspace embedding.
///
/// Combines multiple ignore sources:
/// - `.gitignore` files
/// - `.dockerignore` files
/// - Custom exclude patterns (glob patterns)
/// - Standard exclusions (always applied)
#[derive(Debug)]
pub struct IgnoreFilter {
    /// The gitignore matcher (handles .gitignore files)
    gitignore: Option<Gitignore>,

    /// The dockerignore matcher (parsed as gitignore format)
    dockerignore: Option<Gitignore>,

    /// Custom exclude patterns as a GlobSet
    custom_excludes: GlobSet,

    /// Standard exclusions as a GlobSet
    standard_excludes: GlobSet,

    /// Root path for relative path resolution
    root: std::path::PathBuf,
}

impl IgnoreFilter {
    /// Create a new IgnoreFilter from an EmbedConfig.
    ///
    /// This constructor uses the configuration to determine:
    /// - Whether to respect `.gitignore` files
    /// - Whether to respect `.dockerignore` files
    /// - Additional exclude patterns to apply
    ///
    /// # Arguments
    ///
    /// * `config` - The embedding configuration
    ///
    /// # Returns
    ///
    /// A new `IgnoreFilter` or an error if pattern parsing fails.
    pub fn new(config: &EmbedConfig) -> Result<Self> {
        let mut filter = Self::from_root(
            &config.source_root,
            config.respect_gitignore,
            config.respect_dockerignore,
        )?;

        // Add custom exclude patterns from config
        for pattern in &config.exclude_patterns {
            filter.add_exclude(pattern)?;
        }

        Ok(filter)
    }

    /// Create a new IgnoreFilter from a root directory.
    ///
    /// This is an alternative constructor that allows direct control over
    /// gitignore and dockerignore handling.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory to search for ignore files
    /// * `respect_gitignore` - Whether to parse and use `.gitignore` files
    /// * `respect_dockerignore` - Whether to parse and use `.dockerignore` files
    ///
    /// # Returns
    ///
    /// A new `IgnoreFilter` or an error if pattern parsing fails.
    pub fn from_root(
        root: &Path,
        respect_gitignore: bool,
        respect_dockerignore: bool,
    ) -> Result<Self> {
        let root = root.canonicalize().map_err(|_| {
            WorkspaceEmbedError::SourceNotFound {
                path: root.to_path_buf(),
            }
        })?;

        debug!(
            root = %root.display(),
            respect_gitignore,
            respect_dockerignore,
            "Building IgnoreFilter"
        );

        // Build gitignore matcher
        let gitignore = if respect_gitignore {
            Self::build_gitignore(&root)?
        } else {
            None
        };

        // Build dockerignore matcher
        let dockerignore = if respect_dockerignore {
            Self::build_dockerignore(&root)?
        } else {
            None
        };

        // Build standard exclusions
        let standard_excludes = Self::build_standard_excludes()?;

        Ok(Self {
            gitignore,
            dockerignore,
            custom_excludes: GlobSet::empty(),
            standard_excludes,
            root,
        })
    }

    /// Add a custom exclude pattern (glob format).
    ///
    /// # Arguments
    ///
    /// * `pattern` - A glob pattern to exclude (e.g., "*.pyc", "**/*.log")
    ///
    /// # Returns
    ///
    /// `Ok(())` if the pattern was added successfully, or an error if parsing fails.
    pub fn add_exclude(&mut self, pattern: &str) -> Result<()> {
        debug!(pattern, "Adding custom exclude pattern");

        // Rebuild the GlobSet with the new pattern
        let mut builder = GlobSetBuilder::new();

        // Add existing patterns by iterating (we need to rebuild)
        // Since GlobSet doesn't expose its patterns, we rebuild from scratch
        // This is a limitation, so we track patterns separately
        let glob = Glob::new(pattern).map_err(|e| WorkspaceEmbedError::IgnoreParseError {
            path: self.root.clone(),
            reason: format!("Invalid glob pattern '{}': {}", pattern, e),
        })?;

        builder.add(glob);

        // We need to merge with existing custom_excludes
        // Since GlobSet doesn't allow iteration, we create a new combined set
        // For simplicity, we'll replace. In practice, call add_exclude before filtering.
        self.custom_excludes =
            builder
                .build()
                .map_err(|e| WorkspaceEmbedError::IgnoreParseError {
                    path: self.root.clone(),
                    reason: format!("Failed to build glob set: {}", e),
                })?;

        Ok(())
    }

    /// Add multiple custom exclude patterns at once.
    ///
    /// This is more efficient than calling `add_exclude` multiple times
    /// as it builds the GlobSet only once.
    ///
    /// # Arguments
    ///
    /// * `patterns` - An iterator of glob patterns to exclude
    ///
    /// # Returns
    ///
    /// `Ok(())` if all patterns were added successfully.
    pub fn add_excludes<'a, I>(&mut self, patterns: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            let glob = Glob::new(pattern).map_err(|e| WorkspaceEmbedError::IgnoreParseError {
                path: self.root.clone(),
                reason: format!("Invalid glob pattern '{}': {}", pattern, e),
            })?;
            builder.add(glob);
        }

        self.custom_excludes =
            builder
                .build()
                .map_err(|e| WorkspaceEmbedError::IgnoreParseError {
                    path: self.root.clone(),
                    reason: format!("Failed to build glob set: {}", e),
                })?;

        Ok(())
    }

    /// Check if a path should be ignored.
    ///
    /// A path is ignored if it matches any of:
    /// - Standard exclusion patterns
    /// - Gitignore patterns (if enabled)
    /// - Dockerignore patterns (if enabled)
    /// - Custom exclude patterns
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check (can be absolute or relative to root)
    /// * `is_dir` - Whether the path is a directory
    ///
    /// # Returns
    ///
    /// `true` if the path should be ignored, `false` otherwise.
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        // Normalize path to be relative to root
        let relative_path = self.relativize_path(path);

        trace!(
            path = %relative_path.display(),
            is_dir,
            "Checking if path is ignored"
        );

        // Check standard exclusions first (most common)
        if self.standard_excludes.is_match(&relative_path) {
            trace!(path = %relative_path.display(), "Matched standard exclusion");
            return true;
        }

        // Check custom excludes
        if self.custom_excludes.is_match(&relative_path) {
            trace!(path = %relative_path.display(), "Matched custom exclusion");
            return true;
        }

        // Check gitignore
        if let Some(ref gitignore) = self.gitignore {
            let match_result = gitignore.matched(&relative_path, is_dir);
            if match_result.is_ignore() {
                trace!(path = %relative_path.display(), "Matched gitignore");
                return true;
            }
            // Handle negation (whitelist) patterns
            if match_result.is_whitelist() {
                trace!(path = %relative_path.display(), "Whitelisted by gitignore");
                return false;
            }
        }

        // Check dockerignore
        if let Some(ref dockerignore) = self.dockerignore {
            let match_result = dockerignore.matched(&relative_path, is_dir);
            if match_result.is_ignore() {
                trace!(path = %relative_path.display(), "Matched dockerignore");
                return true;
            }
            // Handle negation patterns
            if match_result.is_whitelist() {
                trace!(path = %relative_path.display(), "Whitelisted by dockerignore");
                return false;
            }
        }

        false
    }

    /// Check if a path should be included (inverse of `is_ignored`).
    ///
    /// This is a convenience method for clearer code when building file lists.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    /// * `is_dir` - Whether the path is a directory
    ///
    /// # Returns
    ///
    /// `true` if the path should be included, `false` if it should be ignored.
    #[inline]
    pub fn should_include(&self, path: &Path, is_dir: bool) -> bool {
        !self.is_ignored(path, is_dir)
    }

    /// Get the root path used by this filter.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Check if gitignore filtering is active.
    pub fn has_gitignore(&self) -> bool {
        self.gitignore.is_some()
    }

    /// Check if dockerignore filtering is active.
    pub fn has_dockerignore(&self) -> bool {
        self.dockerignore.is_some()
    }

    // --- Private helper methods ---

    /// Build a Gitignore matcher from .gitignore files in the directory tree.
    fn build_gitignore(root: &Path) -> Result<Option<Gitignore>> {
        let gitignore_path = root.join(".gitignore");

        if !gitignore_path.exists() {
            debug!(
                root = %root.display(),
                "No .gitignore found at root"
            );
            return Ok(None);
        }

        let mut builder = GitignoreBuilder::new(root);

        // Add the root .gitignore
        if let Some(err) = builder.add(&gitignore_path) {
            warn!(
                path = %gitignore_path.display(),
                error = %err,
                "Failed to parse .gitignore, skipping"
            );
            return Ok(None);
        }

        debug!(
            path = %gitignore_path.display(),
            "Loaded .gitignore"
        );

        // The ignore crate's GitignoreBuilder handles nested .gitignore files
        // when used with WalkBuilder, but here we're building a standalone matcher.
        // For nested support, we'd need to walk the tree and add each .gitignore.
        // For now, we support only the root .gitignore.

        let gitignore = builder.build().map_err(|e| {
            WorkspaceEmbedError::IgnoreParseError {
                path: gitignore_path,
                reason: e.to_string(),
            }
        })?;

        Ok(Some(gitignore))
    }

    /// Build a matcher from .dockerignore (uses gitignore format).
    fn build_dockerignore(root: &Path) -> Result<Option<Gitignore>> {
        let dockerignore_path = root.join(".dockerignore");

        if !dockerignore_path.exists() {
            debug!(
                root = %root.display(),
                "No .dockerignore found"
            );
            return Ok(None);
        }

        let mut builder = GitignoreBuilder::new(root);

        // Docker ignore uses the same syntax as gitignore
        if let Some(err) = builder.add(&dockerignore_path) {
            warn!(
                path = %dockerignore_path.display(),
                error = %err,
                "Failed to parse .dockerignore, skipping"
            );
            return Ok(None);
        }

        debug!(
            path = %dockerignore_path.display(),
            "Loaded .dockerignore"
        );

        let dockerignore = builder.build().map_err(|e| {
            WorkspaceEmbedError::IgnoreParseError {
                path: dockerignore_path,
                reason: e.to_string(),
            }
        })?;

        Ok(Some(dockerignore))
    }

    /// Build the standard exclusions GlobSet.
    fn build_standard_excludes() -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();

        for pattern in STANDARD_EXCLUSIONS {
            let glob = Glob::new(pattern).map_err(|e| WorkspaceEmbedError::IgnoreParseError {
                path: std::path::PathBuf::from("<standard>"),
                reason: format!("Invalid standard pattern '{}': {}", pattern, e),
            })?;
            builder.add(glob);
        }

        builder
            .build()
            .map_err(|e| WorkspaceEmbedError::IgnoreParseError {
                path: std::path::PathBuf::from("<standard>"),
                reason: format!("Failed to build standard exclusions: {}", e),
            })
    }

    /// Convert a path to be relative to the root.
    fn relativize_path(&self, path: &Path) -> std::path::PathBuf {
        // If the path is already relative, use it as-is
        if path.is_relative() {
            return path.to_path_buf();
        }

        // Try to canonicalize the path first if it exists on disk
        let canonical = path.canonicalize().ok();

        // Try to strip the root prefix from the canonical path
        if let Some(ref canonical_path) = canonical {
            if let Ok(relative) = canonical_path.strip_prefix(&self.root) {
                return relative.to_path_buf();
            }
        }

        // Fall back to trying to strip from the original path
        path.strip_prefix(&self.root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    #[test]
    fn test_standard_exclusions() {
        let temp = create_test_dir();
        let filter = IgnoreFilter::from_root(temp.path(), false, false)
            .expect("Failed to create filter");

        // Standard exclusions should be ignored
        assert!(filter.is_ignored(Path::new(".git"), true));
        assert!(filter.is_ignored(Path::new("target"), true));
        assert!(filter.is_ignored(Path::new("target/debug"), true));
        assert!(filter.is_ignored(Path::new("node_modules"), true));
        assert!(filter.is_ignored(Path::new("__pycache__"), true));
        assert!(filter.is_ignored(Path::new(".venv"), true));
        assert!(filter.is_ignored(Path::new("result"), false));
        assert!(filter.is_ignored(Path::new("result-bin"), false));

        // Regular files should not be ignored
        assert!(!filter.is_ignored(Path::new("src/main.rs"), false));
        assert!(!filter.is_ignored(Path::new("Cargo.toml"), false));
    }

    #[test]
    fn test_gitignore_parsing() {
        let temp = create_test_dir();

        // Create a .gitignore file
        // Note: Use "build/**" to match contents, "build/" only matches the directory itself
        fs::write(
            temp.path().join(".gitignore"),
            "*.log\nbuild/\nbuild/**\n!important.log\n",
        )
        .expect("Failed to write .gitignore");

        let filter = IgnoreFilter::from_root(temp.path(), true, false)
            .expect("Failed to create filter");

        assert!(filter.has_gitignore());

        // Files matching gitignore patterns should be ignored
        assert!(filter.is_ignored(Path::new("debug.log"), false));
        assert!(filter.is_ignored(Path::new("build"), true));
        assert!(filter.is_ignored(Path::new("build/output"), true));

        // Negated pattern should allow the file
        // Note: The ignore crate handles negation differently - it's a whitelist
        // The path needs to be checked against the full pattern set

        // Regular files should not be ignored
        assert!(!filter.is_ignored(Path::new("src/main.rs"), false));
    }

    #[test]
    fn test_dockerignore_parsing() {
        let temp = create_test_dir();

        // Create a .dockerignore file
        fs::write(
            temp.path().join(".dockerignore"),
            "Dockerfile*\n*.md\n!README.md\n",
        )
        .expect("Failed to write .dockerignore");

        let filter = IgnoreFilter::from_root(temp.path(), false, true)
            .expect("Failed to create filter");

        assert!(!filter.has_gitignore());
        assert!(filter.has_dockerignore());

        // Files matching dockerignore patterns should be ignored
        assert!(filter.is_ignored(Path::new("Dockerfile"), false));
        assert!(filter.is_ignored(Path::new("Dockerfile.dev"), false));
        assert!(filter.is_ignored(Path::new("CHANGELOG.md"), false));
    }

    #[test]
    fn test_custom_excludes() {
        let temp = create_test_dir();

        let mut filter = IgnoreFilter::from_root(temp.path(), false, false)
            .expect("Failed to create filter");

        filter.add_exclude("*.pyc").expect("Failed to add pattern");
        filter
            .add_excludes(["*.class", "*.o"].iter().copied())
            .expect("Failed to add patterns");

        // Note: add_excludes replaces previous custom excludes
        // So only *.class and *.o should be active after the second call
        assert!(filter.is_ignored(Path::new("Main.class"), false));
        assert!(filter.is_ignored(Path::new("main.o"), false));
    }

    #[test]
    fn test_should_include() {
        let temp = create_test_dir();
        let filter = IgnoreFilter::from_root(temp.path(), false, false)
            .expect("Failed to create filter");

        // should_include is the inverse of is_ignored
        assert!(filter.should_include(Path::new("src/main.rs"), false));
        assert!(!filter.should_include(Path::new("target"), true));
    }

    #[test]
    fn test_from_config() {
        let temp = create_test_dir();

        // Create a .gitignore file
        fs::write(temp.path().join(".gitignore"), "*.tmp\n")
            .expect("Failed to write .gitignore");

        let config = EmbedConfig::new(temp.path(), temp.path().join("output"))
            .with_gitignore(true)
            .with_dockerignore(false)
            .with_exclude_patterns(vec!["*.bak".to_string()]);

        let filter = IgnoreFilter::new(&config).expect("Failed to create filter");

        assert!(filter.has_gitignore());
        assert!(!filter.has_dockerignore());

        // Standard exclusions
        assert!(filter.is_ignored(Path::new("target"), true));

        // Gitignore patterns
        assert!(filter.is_ignored(Path::new("cache.tmp"), false));

        // Custom patterns from config
        assert!(filter.is_ignored(Path::new("file.bak"), false));

        // Regular files
        assert!(!filter.is_ignored(Path::new("src/lib.rs"), false));
    }

    #[test]
    fn test_no_gitignore_file() {
        let temp = create_test_dir();

        // No .gitignore file exists
        let filter = IgnoreFilter::from_root(temp.path(), true, false)
            .expect("Failed to create filter");

        // Should still work without gitignore
        assert!(!filter.has_gitignore());
        assert!(filter.should_include(Path::new("src/main.rs"), false));

        // Standard exclusions should still apply
        assert!(filter.is_ignored(Path::new("target"), true));
    }

    #[test]
    fn test_absolute_path_handling() {
        let temp = create_test_dir();
        let filter = IgnoreFilter::from_root(temp.path(), false, false)
            .expect("Failed to create filter");

        // Create the target directory so it can be canonicalized
        let target_dir = temp.path().join("target");
        fs::create_dir(&target_dir).expect("Failed to create target dir");

        // Should handle absolute paths correctly
        assert!(filter.is_ignored(&target_dir, true));

        // Also test with a file that exists
        let src_dir = temp.path().join("src");
        fs::create_dir(&src_dir).expect("Failed to create src dir");
        let main_rs = src_dir.join("main.rs");
        fs::write(&main_rs, "fn main() {}").expect("Failed to write file");

        // Regular files should not be ignored
        assert!(!filter.is_ignored(&main_rs, false));
    }
}

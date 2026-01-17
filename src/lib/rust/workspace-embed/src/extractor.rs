//! Runtime extraction of embedded workspace files.
//!
//! This module provides the [`ExtractedWorkspace`] struct for extracting files
//! from a `rust_embed::Embed` asset at runtime. Files can be extracted to a
//! temporary directory (cleaned up on drop) or a persistent target directory.
//!
//! # Example
//!
//! ```rust,ignore
//! use workspace_embed::ExtractedWorkspace;
//! use rust_embed::Embed;
//!
//! #[derive(Embed)]
//! #[folder = "embedded/"]
//! struct Assets;
//!
//! // Extract to temp directory (cleaned up on drop)
//! let workspace = ExtractedWorkspace::extract::<Assets>()?;
//!
//! // Access files
//! let flake_path = workspace.path("flake.nix");
//! if workspace.exists("flake.nix") {
//!     // Use the flake
//! }
//!
//! // Optionally persist (prevents cleanup on drop)
//! let permanent_path = workspace.persist();
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use rust_embed::Embed;
use tempfile::TempDir;
use tracing::{debug, trace};

use crate::error::{Result, WorkspaceEmbedError};

/// Manages extracted workspace files at runtime.
///
/// When created via [`extract()`](Self::extract), files are extracted to a
/// temporary directory that is automatically cleaned up when the struct is dropped.
/// Use [`persist()`](Self::persist) to prevent cleanup and retain the files.
///
/// When created via [`extract_to()`](Self::extract_to), files are extracted to
/// the specified target directory and cleanup is never performed.
pub struct ExtractedWorkspace {
    /// Temporary directory handle. If Some, will be cleaned up on drop.
    /// If None (due to persist() or extract_to()), no cleanup occurs.
    temp_dir: Option<TempDir>,
    /// Root path of the extracted workspace.
    root_path: PathBuf,
    /// Number of files extracted.
    file_count: usize,
}

impl ExtractedWorkspace {
    /// Extract all files from a rust-embed asset to a temporary directory.
    ///
    /// The temporary directory will be automatically cleaned up when the
    /// `ExtractedWorkspace` is dropped. Use [`persist()`](Self::persist) to
    /// prevent cleanup.
    ///
    /// # Type Parameters
    ///
    /// * `E` - A type implementing `rust_embed::Embed`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to create temporary directory
    /// - Failed to create parent directories for a file
    /// - Failed to write a file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_embed::ExtractedWorkspace;
    /// use rust_embed::Embed;
    ///
    /// #[derive(Embed)]
    /// #[folder = "embedded/"]
    /// struct Assets;
    ///
    /// let workspace = ExtractedWorkspace::extract::<Assets>()?;
    /// println!("Extracted {} files to {}", workspace.file_count(), workspace.root().display());
    /// ```
    pub fn extract<E: Embed>() -> Result<Self> {
        debug!("Creating temporary directory for workspace extraction");
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path().to_path_buf();

        debug!(path = %root_path.display(), "Extracting workspace to temporary directory");
        let file_count = Self::extract_files::<E>(&root_path)?;

        debug!(file_count, "Extraction complete");
        Ok(Self {
            temp_dir: Some(temp_dir),
            root_path,
            file_count,
        })
    }

    /// Extract all files from a rust-embed asset to a specific directory.
    ///
    /// Unlike [`extract()`](Self::extract), this method does not use a temporary
    /// directory and the extracted files will persist after the struct is dropped.
    ///
    /// # Arguments
    ///
    /// * `target` - The target directory to extract files to. Will be created if
    ///   it doesn't exist.
    ///
    /// # Type Parameters
    ///
    /// * `E` - A type implementing `rust_embed::Embed`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to create target directory
    /// - Failed to create parent directories for a file
    /// - Failed to write a file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_embed::ExtractedWorkspace;
    /// use rust_embed::Embed;
    /// use std::path::Path;
    ///
    /// #[derive(Embed)]
    /// #[folder = "embedded/"]
    /// struct Assets;
    ///
    /// let workspace = ExtractedWorkspace::extract_to::<Assets>(Path::new("/tmp/my-workspace"))?;
    /// // Files persist after workspace is dropped
    /// ```
    pub fn extract_to<E: Embed>(target: impl AsRef<Path>) -> Result<Self> {
        let root_path = target.as_ref().to_path_buf();

        debug!(path = %root_path.display(), "Extracting workspace to target directory");

        // Create target directory if it doesn't exist
        if !root_path.exists() {
            fs::create_dir_all(&root_path)?;
        }

        let file_count = Self::extract_files::<E>(&root_path)?;

        debug!(file_count, "Extraction complete");
        Ok(Self {
            temp_dir: None, // No temp dir, no cleanup
            root_path,
            file_count,
        })
    }

    /// Internal method to extract all files from an Embed asset to a directory.
    fn extract_files<E: Embed>(root_path: &Path) -> Result<usize> {
        let mut file_count = 0;

        for file_path in E::iter() {
            let relative_path = file_path.as_ref();
            trace!(path = %relative_path, "Extracting file");

            // Get the file content
            let embedded_file = E::get(relative_path).ok_or_else(|| {
                WorkspaceEmbedError::ExtractionError {
                    reason: format!("File listed in iter() but not found via get(): {}", relative_path),
                }
            })?;

            // Construct the full path
            let full_path = root_path.join(relative_path);

            // Create parent directories if needed
            if let Some(parent) = full_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            // Write the file content
            fs::write(&full_path, embedded_file.data.as_ref())?;

            file_count += 1;
        }

        Ok(file_count)
    }

    /// Get the root path of the extracted workspace.
    ///
    /// All relative paths passed to other methods are resolved against this root.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let workspace = ExtractedWorkspace::extract::<Assets>()?;
    /// println!("Workspace root: {}", workspace.root().display());
    /// ```
    #[inline]
    pub fn root(&self) -> &Path {
        &self.root_path
    }

    /// Get the absolute path for a relative path within the workspace.
    ///
    /// This simply joins the relative path to the workspace root. It does not
    /// check whether the file exists.
    ///
    /// # Arguments
    ///
    /// * `relative` - A path relative to the workspace root
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let workspace = ExtractedWorkspace::extract::<Assets>()?;
    /// let flake_path = workspace.path("flake.nix");
    /// let nested_path = workspace.path("charts/myapp/Chart.yaml");
    /// ```
    #[inline]
    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root_path.join(relative)
    }

    /// Check if a file or directory exists at the given relative path.
    ///
    /// # Arguments
    ///
    /// * `relative` - A path relative to the workspace root
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let workspace = ExtractedWorkspace::extract::<Assets>()?;
    /// if workspace.exists("flake.nix") {
    ///     println!("Found flake.nix!");
    /// }
    /// ```
    #[inline]
    pub fn exists(&self, relative: impl AsRef<Path>) -> bool {
        self.path(relative).exists()
    }

    /// Check if a .git directory exists in the workspace root.
    ///
    /// This is useful for Nix flake operations which require a git repository
    /// to properly resolve flake references.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let workspace = ExtractedWorkspace::extract::<Assets>()?;
    /// if workspace.has_git() {
    ///     println!(".git directory present for Nix flake resolution");
    /// }
    /// ```
    #[inline]
    pub fn has_git(&self) -> bool {
        self.path(".git").is_dir()
    }

    /// Get the path to the .git directory if it exists.
    ///
    /// # Returns
    ///
    /// `Some(PathBuf)` containing the path to .git if it exists as a directory,
    /// `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let workspace = ExtractedWorkspace::extract::<Assets>()?;
    /// if let Some(git_dir) = workspace.git_dir() {
    ///     println!("Git directory at: {}", git_dir.display());
    /// }
    /// ```
    pub fn git_dir(&self) -> Option<PathBuf> {
        let git_path = self.path(".git");
        if git_path.is_dir() {
            Some(git_path)
        } else {
            None
        }
    }

    /// Get the number of files that were extracted.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let workspace = ExtractedWorkspace::extract::<Assets>()?;
    /// println!("Extracted {} files", workspace.file_count());
    /// ```
    #[inline]
    pub fn file_count(&self) -> usize {
        self.file_count
    }

    /// Prevent cleanup on drop and return the path to the extracted workspace.
    ///
    /// After calling this method, the extracted files will persist after the
    /// `ExtractedWorkspace` is dropped. This is useful when you want to keep
    /// the extracted workspace for later use.
    ///
    /// # Returns
    ///
    /// The path to the extracted workspace root.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let workspace = ExtractedWorkspace::extract::<Assets>()?;
    /// let path = workspace.persist();
    /// println!("Workspace persisted at: {}", path.display());
    /// // Files remain after this point
    /// ```
    pub fn persist(mut self) -> PathBuf {
        if let Some(temp_dir) = self.temp_dir.take() {
            // Get the path before keeping the directory
            let path = temp_dir.path().to_path_buf();
            // Prevent cleanup by "keeping" the temp directory
            let _ = temp_dir.keep();
            debug!(path = %path.display(), "Workspace persisted, cleanup disabled");
            path
        } else {
            // Already persistent (created via extract_to)
            debug!(path = %self.root_path.display(), "Workspace already persistent");
            self.root_path.clone()
        }
    }
}

impl std::fmt::Debug for ExtractedWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtractedWorkspace")
            .field("root_path", &self.root_path)
            .field("file_count", &self.file_count)
            .field("is_temporary", &self.temp_dir.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_embed::Embed;
    use std::fs;

    // A minimal test embed with no files (for testing the extraction logic)
    #[derive(Embed)]
    #[folder = "src/"] // Embed the src directory itself for testing
    struct TestAssets;

    #[test]
    fn test_extract_creates_temp_dir() {
        let workspace = ExtractedWorkspace::extract::<TestAssets>().unwrap();
        assert!(workspace.root().exists());
        assert!(workspace.file_count() > 0);
    }

    #[test]
    fn test_extract_to_creates_target_dir() {
        let target = tempfile::tempdir().unwrap();
        let target_path = target.path().join("test-workspace");

        let workspace = ExtractedWorkspace::extract_to::<TestAssets>(&target_path).unwrap();

        assert!(target_path.exists());
        assert_eq!(workspace.root(), target_path);
        assert!(workspace.file_count() > 0);
    }

    #[test]
    fn test_path_resolution() {
        let workspace = ExtractedWorkspace::extract::<TestAssets>().unwrap();

        let expected = workspace.root().join("lib.rs");
        assert_eq!(workspace.path("lib.rs"), expected);

        let nested = workspace.root().join("nested/file.rs");
        assert_eq!(workspace.path("nested/file.rs"), nested);
    }

    #[test]
    fn test_exists() {
        let workspace = ExtractedWorkspace::extract::<TestAssets>().unwrap();

        // lib.rs should exist since we embedded the src directory
        assert!(workspace.exists("lib.rs"));

        // Non-existent file should not exist
        assert!(!workspace.exists("nonexistent.rs"));
    }

    #[test]
    fn test_persist_prevents_cleanup() {
        let path = {
            let workspace = ExtractedWorkspace::extract::<TestAssets>().unwrap();
            workspace.persist()
        };

        // After dropping, the path should still exist
        assert!(path.exists());

        // Clean up manually
        fs::remove_dir_all(&path).unwrap();
    }

    #[test]
    fn test_temp_dir_cleaned_up_on_drop() {
        let path = {
            let workspace = ExtractedWorkspace::extract::<TestAssets>().unwrap();
            workspace.root().to_path_buf()
        };

        // After dropping, the temp dir should be cleaned up
        assert!(!path.exists());
    }

    #[test]
    fn test_debug_impl() {
        let workspace = ExtractedWorkspace::extract::<TestAssets>().unwrap();
        let debug_str = format!("{:?}", workspace);

        assert!(debug_str.contains("ExtractedWorkspace"));
        assert!(debug_str.contains("root_path"));
        assert!(debug_str.contains("file_count"));
        assert!(debug_str.contains("is_temporary"));
    }
}

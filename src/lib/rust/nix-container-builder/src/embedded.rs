//! Embedded workspace management
//!
//! This module provides access to the Nix files embedded at compile time.

use crate::error::{NixContainerError, Result};
use rust_embed::Embed;
use std::path::PathBuf;
use tracing::info;

/// Embedded workspace files
#[derive(Embed)]
#[folder = "embedded/"]
pub struct EmbeddedAssets;

/// Re-export ExtractedWorkspace from workspace-embed
pub use workspace_embed::ExtractedWorkspace;

/// Extract the embedded workspace for Nix container builds
///
/// This extracts all embedded files to a temporary directory and
/// returns a handle to the workspace. The workspace is cleaned up
/// when the handle is dropped.
pub fn extract_embedded() -> Result<ExtractedWorkspace> {
    info!("Extracting embedded workspace...");

    let workspace = workspace_embed::ExtractedWorkspace::extract::<EmbeddedAssets>()
        .map_err(|e| NixContainerError::Other(format!("Failed to extract workspace: {}", e)))?;

    info!(
        "Extracted {} files to {}",
        workspace.file_count(),
        workspace.root().display()
    );

    // Verify critical files exist
    if !workspace.exists("flake.nix") {
        return Err(NixContainerError::Other(
            "Root flake.nix not found in extracted workspace".to_string(),
        ));
    }

    if workspace.has_git() {
        info!("Minimal .git directory present for Nix flake resolution");
    } else {
        tracing::warn!("No .git directory - Nix flake path imports may fail");
    }

    Ok(workspace)
}

/// Get the path to the containers directory within an extracted workspace
pub fn containers_dir(workspace: &ExtractedWorkspace) -> PathBuf {
    workspace.path("src/containers/firestream")
}

/// List all available container names from an extracted workspace
///
/// Scans the containers directory and returns names of all directories
/// that contain a flake.nix file.
pub fn list_containers(workspace: &ExtractedWorkspace) -> Result<Vec<String>> {
    let mut containers = Vec::new();
    let containers_path = containers_dir(workspace);

    if !containers_path.exists() {
        tracing::debug!(
            "Containers directory does not exist: {}",
            containers_path.display()
        );
        return Ok(containers);
    }

    for entry in std::fs::read_dir(&containers_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check if this directory contains a flake.nix
            if path.join("flake.nix").exists() {
                if let Some(name) = entry.file_name().to_str() {
                    containers.push(name.to_string());
                }
            }
        }
    }

    containers.sort();
    Ok(containers)
}

/// Check if a specific container exists in an extracted workspace
pub fn has_container(workspace: &ExtractedWorkspace, name: &str) -> bool {
    let container_path = containers_dir(workspace).join(name);
    container_path.join("flake.nix").exists()
}

/// Get the path to a specific container in an extracted workspace
pub fn container_path(workspace: &ExtractedWorkspace, name: &str) -> Option<PathBuf> {
    let path = containers_dir(workspace).join(name);
    if path.join("flake.nix").exists() {
        Some(path)
    } else {
        None
    }
}

/// Get count of embedded files
pub fn embedded_file_count() -> usize {
    EmbeddedAssets::iter().count()
}

/// List all embedded file paths
pub fn list_embedded_files() -> Vec<String> {
    EmbeddedAssets::iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_file_count() {
        let count = embedded_file_count();
        // Should have at least the root flake files
        assert!(
            count >= 2,
            "Expected at least 2 embedded files, got {}",
            count
        );
    }

    #[test]
    fn test_list_embedded_files() {
        let files = list_embedded_files();
        assert!(
            !files.is_empty(),
            "Expected embedded files list to be non-empty"
        );

        // Check that root flake.nix is present
        assert!(
            files.iter().any(|f| f == "flake.nix"),
            "Expected flake.nix in embedded files"
        );
    }
}

//! Docker image loading utilities
//!
//! This module handles loading built container images into Docker.

use crate::error::{NixContainerError, Result};
use std::path::Path;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Result of loading an image into Docker
#[derive(Debug, Clone)]
pub struct LoadResult {
    /// Full image name (repository:tag)
    pub image_name: String,

    /// Image tag
    pub image_tag: String,

    /// Docker image ID (sha256 hash)
    pub image_id: Option<String>,
}

impl LoadResult {
    /// Get the full image reference (name:tag)
    pub fn full_ref(&self) -> String {
        format!("{}:{}", self.image_name, self.image_tag)
    }
}

impl std::fmt::Display for LoadResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.image_name, self.image_tag)?;
        if let Some(id) = &self.image_id {
            write!(f, " ({})", &id[..12.min(id.len())])?;
        }
        Ok(())
    }
}

/// Docker image loader
pub struct DockerLoader;

impl DockerLoader {
    /// Create a new DockerLoader
    pub fn new() -> Self {
        Self
    }

    /// Load a container image tarball into Docker
    pub async fn load_image(&self, tarball: &Path) -> Result<LoadResult> {
        info!("Loading image from {}", tarball.display());

        if !tarball.exists() {
            return Err(NixContainerError::DockerLoadFailed(format!(
                "Tarball does not exist: {}",
                tarball.display()
            )));
        }

        let output = Command::new("docker")
            .args(["load", "-i", tarball.to_str().unwrap()])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Docker load failed: {}", stderr);
            return Err(NixContainerError::DockerLoadFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Docker load output: {}", stdout);

        self.parse_load_output(&stdout)
    }

    /// Parse the output of `docker load` to extract image info
    fn parse_load_output(&self, output: &str) -> Result<LoadResult> {
        // Docker load output looks like:
        // "Loaded image: firestream-redis:7.0"
        // or: "Loaded image ID: sha256:abc123..."

        for line in output.lines() {
            let line = line.trim();

            // Parse "Loaded image: name:tag"
            if let Some(rest) = line.strip_prefix("Loaded image:") {
                let rest = rest.trim();
                if let Some((name, tag)) = rest.rsplit_once(':') {
                    return Ok(LoadResult {
                        image_name: name.to_string(),
                        image_tag: tag.to_string(),
                        image_id: None,
                    });
                }
            }

            // Parse "Loaded image ID: sha256:..."
            if let Some(rest) = line.strip_prefix("Loaded image ID:") {
                let image_id = rest.trim().to_string();
                return Ok(LoadResult {
                    image_name: "unknown".to_string(),
                    image_tag: "latest".to_string(),
                    image_id: Some(image_id),
                });
            }
        }

        // Fallback: try to find any image reference in the output
        for line in output.lines() {
            if line.contains(':') && !line.contains("sha256:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.contains(':') && !part.ends_with(':') {
                        if let Some((name, tag)) = part.rsplit_once(':') {
                            return Ok(LoadResult {
                                image_name: name.to_string(),
                                image_tag: tag.to_string(),
                                image_id: None,
                            });
                        }
                    }
                }
            }
        }

        Err(NixContainerError::DockerLoadFailed(
            "Could not parse docker load output".to_string(),
        ))
    }

    /// Tag an existing image with a new name
    pub async fn tag_image(&self, source: &str, target: &str) -> Result<()> {
        info!("Tagging {} as {}", source, target);

        let output = Command::new("docker")
            .args(["tag", source, target])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NixContainerError::DockerLoadFailed(format!(
                "Failed to tag image: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Check if an image exists locally
    pub async fn image_exists(&self, name: &str, tag: &str) -> Result<bool> {
        let image_ref = format!("{}:{}", name, tag);

        let output = Command::new("docker")
            .args(["image", "inspect", &image_ref])
            .output()
            .await?;

        Ok(output.status.success())
    }

    /// Get the ID of an image
    pub async fn get_image_id(&self, name: &str, tag: &str) -> Result<Option<String>> {
        let image_ref = format!("{}:{}", name, tag);

        let output = Command::new("docker")
            .args(["image", "inspect", "--format", "{{.Id}}", &image_ref])
            .output()
            .await?;

        if output.status.success() {
            let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    /// Remove a local image
    pub async fn remove_image(&self, name: &str, tag: &str) -> Result<()> {
        let image_ref = format!("{}:{}", name, tag);

        let output = Command::new("docker")
            .args(["rmi", &image_ref])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NixContainerError::DockerLoadFailed(format!(
                "Failed to remove image: {}",
                stderr
            )));
        }

        Ok(())
    }
}

impl Default for DockerLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_load_output() {
        let loader = DockerLoader::new();

        let output1 = "Loaded image: firestream-redis:7.0\n";
        let result1 = loader.parse_load_output(output1).unwrap();
        assert_eq!(result1.image_name, "firestream-redis");
        assert_eq!(result1.image_tag, "7.0");

        let output2 = "Loaded image ID: sha256:abc123def456\n";
        let result2 = loader.parse_load_output(output2).unwrap();
        assert_eq!(result2.image_id, Some("sha256:abc123def456".to_string()));
    }

    #[test]
    fn test_load_result_display() {
        let result = LoadResult {
            image_name: "firestream-redis".to_string(),
            image_tag: "7.0".to_string(),
            image_id: Some("sha256:abc123def456".to_string()),
        };
        assert!(result.to_string().contains("firestream-redis:7.0"));
        // The display truncates the ID to first 12 chars
        assert!(result.to_string().contains("sha256:abc12"));
    }
}

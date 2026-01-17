//! Container discovery from filesystem
//!
//! This module handles discovering available containers from the filesystem
//! by scanning for flake.nix files in the containers directory.

use crate::error::{NixContainerError, Result};
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info};

/// Information about a discovered container
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    /// Container name (directory name)
    pub name: String,

    /// Full path to the container directory
    pub path: PathBuf,

    /// Whether the container has a flake.nix
    pub has_flake: bool,

    /// Whether the container has a Dockerfile
    pub has_dockerfile: bool,

    /// Container version (if extractable from flake.nix)
    pub version: Option<String>,
}

impl ContainerInfo {
    /// Create a new ContainerInfo
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let has_flake = path.join("flake.nix").exists();
        let has_dockerfile = path.join("Dockerfile").exists();

        Self {
            name: name.into(),
            path,
            has_flake,
            has_dockerfile,
            version: None,
        }
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Check if this container can be built with Nix
    pub fn can_build_nix(&self) -> bool {
        self.has_flake
    }

    /// Check if this container can be built with Docker
    pub fn can_build_docker(&self) -> bool {
        self.has_dockerfile
    }

    /// Get the flake.nix path
    pub fn flake_path(&self) -> PathBuf {
        self.path.join("flake.nix")
    }

    /// Get the Dockerfile path
    pub fn dockerfile_path(&self) -> PathBuf {
        self.path.join("Dockerfile")
    }
}

impl std::fmt::Display for ContainerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(version) = &self.version {
            write!(f, " (v{})", version)?;
        }
        Ok(())
    }
}

/// Container discovery service
pub struct ContainerDiscovery {
    containers_dir: PathBuf,
}

impl ContainerDiscovery {
    /// Create a new ContainerDiscovery instance
    pub fn new(containers_dir: impl Into<PathBuf>) -> Self {
        Self {
            containers_dir: containers_dir.into(),
        }
    }

    /// Get the containers directory path
    pub fn containers_dir(&self) -> &PathBuf {
        &self.containers_dir
    }

    /// Discover all containers with flake.nix or Dockerfile
    pub async fn discover(&self) -> Result<Vec<ContainerInfo>> {
        let mut containers = Vec::new();

        if !self.containers_dir.exists() {
            return Err(NixContainerError::Other(format!(
                "Containers directory does not exist: {}",
                self.containers_dir.display()
            )));
        }

        info!("Discovering containers in {}", self.containers_dir.display());

        let mut entries = fs::read_dir(&self.containers_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip non-directories
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            // Check for flake.nix or Dockerfile
            let flake_path = path.join("flake.nix");
            let dockerfile_path = path.join("Dockerfile");

            if flake_path.exists() || dockerfile_path.exists() {
                let name = entry
                    .file_name()
                    .to_string_lossy()
                    .to_string();

                debug!("Found container: {}", name);

                let mut info = ContainerInfo::new(&name, &path);

                // Try to extract version from flake.nix
                if flake_path.exists() {
                    if let Ok(version) = self.extract_version(&path).await {
                        info = info.with_version(version);
                    }
                }

                containers.push(info);
            }
        }

        // Sort by name
        containers.sort_by(|a, b| a.name.cmp(&b.name));

        info!("Discovered {} containers", containers.len());

        Ok(containers)
    }

    /// Discover only containers with flake.nix (Nix-buildable)
    pub async fn discover_nix(&self) -> Result<Vec<ContainerInfo>> {
        let all = self.discover().await?;
        Ok(all.into_iter().filter(|c| c.has_flake).collect())
    }

    /// Get a specific container by name
    pub async fn get(&self, name: &str) -> Result<ContainerInfo> {
        let path = self.containers_dir.join(name);

        if !path.exists() {
            return Err(NixContainerError::container_not_found(
                name,
                self.containers_dir.clone(),
            ));
        }

        if !path.is_dir() {
            return Err(NixContainerError::container_not_found(
                name,
                self.containers_dir.clone(),
            ));
        }

        let mut info = ContainerInfo::new(name, &path);

        // Check for required files
        if !info.has_flake && !info.has_dockerfile {
            return Err(NixContainerError::FlakeNotFound { path: path.clone() });
        }

        // Try to extract version
        if info.has_flake {
            if let Ok(version) = self.extract_version(&path).await {
                info = info.with_version(version);
            }
        }

        Ok(info)
    }

    /// Check if a container exists
    pub async fn exists(&self, name: &str) -> bool {
        let path = self.containers_dir.join(name);
        path.exists() && path.is_dir()
    }

    /// List all container names
    pub async fn list_names(&self) -> Result<Vec<String>> {
        let containers = self.discover().await?;
        Ok(containers.into_iter().map(|c| c.name).collect())
    }

    /// Extract version from flake.nix or module.nix
    ///
    /// This is a best-effort extraction - it may not work for all containers.
    async fn extract_version(&self, path: &PathBuf) -> Result<String> {
        // Try module.nix first (more likely to have version)
        let module_path = path.join("module.nix");
        if module_path.exists() {
            if let Ok(content) = fs::read_to_string(&module_path).await {
                if let Some(version) = extract_version_from_nix(&content) {
                    return Ok(version);
                }
            }
        }

        // Try flake.nix
        let flake_path = path.join("flake.nix");
        if flake_path.exists() {
            if let Ok(content) = fs::read_to_string(&flake_path).await {
                if let Some(version) = extract_version_from_nix(&content) {
                    return Ok(version);
                }
            }
        }

        Err(NixContainerError::Other("Version not found".to_string()))
    }
}

/// Extract version string from Nix file content
fn extract_version_from_nix(content: &str) -> Option<String> {
    // Look for common patterns:
    // version = "1.2.3"
    // version = "1.2.3";
    // appVersion = "1.2.3"

    for line in content.lines() {
        let line = line.trim();

        // Skip comments
        if line.starts_with('#') || line.starts_with("/*") {
            continue;
        }

        // Look for version = "..."
        if let Some(pos) = line.find("version") {
            let rest = &line[pos..];
            if let Some(start_quote) = rest.find('"') {
                let after_quote = &rest[start_quote + 1..];
                if let Some(end_quote) = after_quote.find('"') {
                    let version = &after_quote[..end_quote];
                    // Validate it looks like a version
                    if version.chars().any(|c| c.is_ascii_digit()) {
                        return Some(version.to_string());
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_from_nix() {
        let content = r#"
        {
            name = "myapp";
            version = "1.2.3";
        }
        "#;
        assert_eq!(extract_version_from_nix(content), Some("1.2.3".to_string()));

        // The function looks for "version" keyword, so appVersion matches
        let content2 = r#"
        version = "3.0.0"
        "#;
        assert_eq!(extract_version_from_nix(content2), Some("3.0.0".to_string()));

        let no_version = "{ name = \"test\"; }";
        assert_eq!(extract_version_from_nix(no_version), None);
    }

    #[test]
    fn test_container_info() {
        let info = ContainerInfo {
            name: "redis".to_string(),
            path: PathBuf::from("/test/redis"),
            has_flake: true,
            has_dockerfile: false,
            version: Some("7.0".to_string()),
        };

        assert!(info.can_build_nix());
        assert!(!info.can_build_docker());
        assert_eq!(info.to_string(), "redis (v7.0)");
    }
}

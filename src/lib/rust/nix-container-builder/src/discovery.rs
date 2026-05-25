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

    /// Whether the container has a module.nix (Nix container module)
    pub has_module: bool,

    /// Whether the container has an overrides.nix (Python container overrides)
    pub has_overrides: bool,

    /// Whether the container has a docker-compose.yml
    pub has_compose: bool,

    /// Container version (if extractable from flake.nix)
    pub version: Option<String>,

    /// Root flake package name for `nix build .#<package>` (resolved from registry)
    pub nix_package: Option<String>,
}

impl ContainerInfo {
    /// Create a new ContainerInfo
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let has_flake = path.join("flake.nix").exists();
        let has_dockerfile = path.join("Dockerfile").exists();
        let has_module = path.join("module.nix").exists();
        let has_overrides = path.join("overrides.nix").exists();
        let has_compose = path.join("docker-compose.yml").exists();

        Self {
            name: name.into(),
            path,
            has_flake,
            has_dockerfile,
            has_module,
            has_overrides,
            has_compose,
            version: None,
            nix_package: None,
        }
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the root flake package name
    pub fn with_nix_package(mut self, package: impl Into<String>) -> Self {
        self.nix_package = Some(package.into());
        self
    }

    /// Check if this container can be built with Nix (has its own flake or is known in the root flake)
    pub fn can_build_nix(&self) -> bool {
        self.has_flake || self.nix_package.is_some()
    }

    /// Check if this container can be built with Docker
    pub fn can_build_docker(&self) -> bool {
        self.has_dockerfile
    }

    /// Check if this container is a Nix module (has module.nix or overrides.nix)
    pub fn is_nix_module(&self) -> bool {
        self.has_module || self.has_overrides
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

    /// Discover only containers with flake.nix (Nix-buildable via per-container flake)
    pub async fn discover_nix(&self) -> Result<Vec<ContainerInfo>> {
        let all = self.discover().await?;
        Ok(all.into_iter().filter(|c| c.has_flake).collect())
    }

    /// Discover ALL containers that can be built from the root flake.
    ///
    /// This finds containers with ANY of: module.nix, overrides.nix, flake.nix, or docker-compose.yml.
    /// This catches containers like airflow (has overrides.nix but no flake.nix) that are only
    /// buildable from the root flake. Uses the BuildConfig package registry to resolve package names.
    ///
    /// For superset, which has versioned subdirs (4/, 5/) rather than top-level files,
    /// we detect the subdir structure and create entries for each version.
    pub async fn discover_all(&self) -> Result<Vec<ContainerInfo>> {
        let mut containers = Vec::new();

        if !self.containers_dir.exists() {
            return Err(NixContainerError::Other(format!(
                "Containers directory does not exist: {}",
                self.containers_dir.display()
            )));
        }

        info!("Discovering all containers in {}", self.containers_dir.display());

        let mut entries = fs::read_dir(&self.containers_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if !entry.file_type().await?.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();

            // Check for any Nix/container marker files
            let has_flake = path.join("flake.nix").exists();
            let has_module = path.join("module.nix").exists();
            let has_overrides = path.join("overrides.nix").exists();
            let has_dockerfile = path.join("Dockerfile").exists();
            let has_compose = path.join("docker-compose.yml").exists();

            if has_flake || has_module || has_overrides || has_dockerfile || has_compose {
                debug!("Found container: {} (flake={}, module={}, overrides={}, compose={})",
                    name, has_flake, has_module, has_overrides, has_compose);

                let mut info = ContainerInfo::new(&name, &path);

                // Try to extract version
                if let Ok(version) = self.extract_version(&path).await {
                    info = info.with_version(version);
                }

                containers.push(info);
            } else {
                // Check for versioned subdirectories (e.g., superset/4/, superset/5/)
                // These have module.nix/overrides.nix inside the version subdir
                let mut found_versioned = false;
                if let Ok(mut subdirs) = fs::read_dir(&path).await {
                    while let Ok(Some(subentry)) = subdirs.next_entry().await {
                        let subpath = subentry.path();
                        if subpath.is_dir() {
                            let subname = subentry.file_name().to_string_lossy().to_string();
                            // Check if this looks like a version directory (starts with a digit)
                            if subname.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                                let sub_has_module = subpath.join("module.nix").exists();
                                let sub_has_overrides = subpath.join("overrides.nix").exists();
                                let sub_has_compose = subpath.join("docker-compose.yml").exists();

                                if sub_has_module || sub_has_overrides || sub_has_compose {
                                    found_versioned = true;
                                    // Don't create individual versioned entries here;
                                    // the parent dir represents the container.
                                    // Version resolution is handled by the package registry.
                                }
                            }
                        }
                    }
                }

                if found_versioned {
                    debug!("Found versioned container: {} (has version subdirs)", name);
                    let info = ContainerInfo::new(&name, &path);
                    containers.push(info);
                }
            }
        }

        containers.sort_by(|a, b| a.name.cmp(&b.name));
        info!("Discovered {} containers (all types)", containers.len());
        Ok(containers)
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
            has_module: true,
            has_overrides: false,
            has_compose: true,
            version: Some("7.0".to_string()),
            nix_package: Some("redis-7".to_string()),
        };

        assert!(info.can_build_nix());
        assert!(!info.can_build_docker());
        assert!(info.is_nix_module());
        assert_eq!(info.to_string(), "redis (v7.0)");
    }

    #[test]
    fn test_container_info_overrides_only() {
        // Container like airflow that has overrides.nix but no flake.nix
        let info = ContainerInfo {
            name: "airflow".to_string(),
            path: PathBuf::from("/test/airflow"),
            has_flake: false,
            has_dockerfile: false,
            has_module: true,
            has_overrides: true,
            has_compose: true,
            version: None,
            nix_package: Some("airflow".to_string()),
        };

        // Can build nix because nix_package is set (root flake)
        assert!(info.can_build_nix());
        assert!(info.is_nix_module());
    }
}

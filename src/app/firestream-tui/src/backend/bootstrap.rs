//! Bootstrap checker for Docker environment detection.
//!
//! Validates the Docker environment, checks for builder images, Nix cache volumes,
//! and inventories existing container images before the TUI starts.

use super::manifest::{ContainerManifest, ManifestRegistry};
use docker_manager::DockerManager;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Complete state of the Docker environment at startup.
#[derive(Debug, Clone)]
pub struct BootstrapState {
    // Docker
    pub docker_available: bool,
    pub docker_version: Option<String>,
    pub docker_healthy: bool,
    pub docker_warnings: Vec<String>,

    // Builder
    pub builder_image_present: bool,
    pub nix_volume_exists: bool,
    pub nix_volume_name: String,

    // Workspace
    pub repo_root: Option<PathBuf>,
    pub platform_arch: String,

    // Inventory
    pub built_images: HashMap<String, String>,
    pub running_containers: HashMap<String, String>,
    pub available_containers: Vec<ContainerManifest>,

    // Warnings
    pub warnings: Vec<String>,
}

impl Default for BootstrapState {
    fn default() -> Self {
        let arch = std::env::consts::ARCH;
        let volume_name = match arch {
            "x86_64" | "x86" => "firestream-nix-store-amd64",
            "aarch64" | "arm" => "firestream-nix-store-arm64",
            other => return Self {
                nix_volume_name: format!("firestream-nix-store-{}", other),
                platform_arch: arch.to_string(),
                ..Self::empty()
            },
        };
        Self {
            nix_volume_name: volume_name.to_string(),
            platform_arch: arch.to_string(),
            ..Self::empty()
        }
    }
}

impl BootstrapState {
    fn empty() -> Self {
        Self {
            docker_available: false,
            docker_version: None,
            docker_healthy: false,
            docker_warnings: vec![],
            builder_image_present: false,
            nix_volume_exists: false,
            nix_volume_name: String::new(),
            repo_root: None,
            platform_arch: String::new(),
            built_images: HashMap::new(),
            running_containers: HashMap::new(),
            available_containers: vec![],
            warnings: vec![],
        }
    }

    /// Summary line for the splash screen.
    pub fn summary(&self) -> String {
        if !self.docker_available {
            return "Docker not available - running in demo mode".to_string();
        }
        let built = self.built_images.len();
        let total = self.available_containers.len();
        let running = self.running_containers.values()
            .filter(|s| s.as_str() == "running")
            .count();
        format!(
            "Docker {} | {} of {} built | {} running | Nix cache: {}",
            self.docker_version.as_deref().unwrap_or("unknown"),
            built,
            total,
            running,
            if self.nix_volume_exists { "ready" } else { "cold" },
        )
    }
}

/// Checks the Docker environment and inventories available containers.
pub struct BootstrapChecker;

impl BootstrapChecker {
    /// Run all bootstrap checks and return the environment state.
    pub async fn check(repo_root: Option<&Path>) -> BootstrapState {
        let mut state = BootstrapState::default();

        // Detect repo root
        state.repo_root = repo_root.map(|p| p.to_path_buf()).or_else(find_repo_root);

        // Connect to Docker
        let docker = match DockerManager::new().await {
            Ok(d) => d,
            Err(e) => {
                state.docker_available = false;
                state.warnings.push(format!("Docker not available: {}", e));
                // Still discover containers from filesystem
                if let Some(ref root) = state.repo_root {
                    let registry = ManifestRegistry::discover(root);
                    state.available_containers = registry.all().to_vec();
                }
                return state;
            }
        };

        state.docker_available = true;

        // Health check (integrates docker-manager's HealthCheckResult)
        match docker.health_check().await {
            Ok(health) => {
                state.docker_healthy = health.healthy;
                state.docker_warnings = health.warnings.clone();
                if !health.errors.is_empty() {
                    state.warnings.extend(health.errors.iter().map(|e| format!("Docker: {}", e)));
                }
            }
            Err(e) => {
                state.warnings.push(format!("Health check failed: {}", e));
            }
        }

        // Docker version
        match docker.version().await {
            Ok(info) => {
                state.docker_version = info.version;
            }
            Err(_) => {}
        }

        // Check for builder image (nixos/nix:latest)
        state.builder_image_present = docker.get_image("nixos/nix").await.is_ok();

        // Check for Nix store volume
        state.nix_volume_exists = docker.get_volume(&state.nix_volume_name).await.is_ok();

        // Inventory existing firestream images
        if let Ok(images) = docker.list_images(false).await {
            for image in images {
                for tag in &image.repo_tags {
                    if tag.starts_with("firestream-") {
                        if let Some((name, version)) = tag.split_once(':') {
                            let container_name = name.strip_prefix("firestream-").unwrap_or(name);
                            state.built_images.insert(container_name.to_string(), version.to_string());
                        }
                    }
                }
            }
        }

        // Check for stale build locks
        if let Some(ref root) = state.repo_root {
            let lock_path = root.join("_build/.build-batch.lock");
            if lock_path.exists() {
                state.warnings.push("Stale build lock detected (_build/.build-batch.lock)".to_string());
            }
        }

        // Check disk space
        if let Ok(usage) = docker.disk_usage().await {
            let total_bytes: i64 = usage.images.as_ref()
                .map(|imgs| imgs.iter().map(|i| i.size).sum())
                .unwrap_or(0);
            let total_gb = total_bytes as f64 / 1_073_741_824.0;
            if total_gb > 50.0 {
                state.warnings.push(format!("Docker images using {:.1} GB disk space", total_gb));
            }
        }

        // Discover available containers
        if let Some(ref root) = state.repo_root {
            let registry = ManifestRegistry::discover(root);
            state.available_containers = registry.all().to_vec();
        }

        // Inventory running firestream containers
        if let Ok(containers) = docker.list_containers(true).await {
            for c in containers {
                let names = c.names.as_ref().map(|n| n.join(", ")).unwrap_or_default();
                for manifest in &state.available_containers {
                    if names.contains(&format!("firestream-{}", manifest.name)) {
                        let container_state = c.state.as_deref().unwrap_or("unknown").to_string();
                        state.running_containers.insert(manifest.name.clone(), container_state);
                    }
                }
            }
        }

        state
    }
}

/// Find the repository root by walking up from the current directory.
fn find_repo_root() -> Option<PathBuf> {
    let current = std::env::current_dir().ok()?;
    let mut dir = current.as_path();
    loop {
        if dir.join("flake.nix").exists() && dir.join("src/containers/firestream").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

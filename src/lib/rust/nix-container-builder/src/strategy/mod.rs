//! Build strategy definitions and implementations
//!
//! This module provides the strategy pattern for building containers,
//! with different implementations for native Nix builds and Docker-based builds.

mod docker;
mod native;

pub use docker::DockerNixStrategy;
pub use native::NativeNixStrategy;

use crate::config::BuildConfig;
use crate::discovery::ContainerInfo;
use crate::error::Result;
use crate::platform::PlatformInfo;
use crate::progress::BuildProgress;
use async_trait::async_trait;
use std::path::PathBuf;

/// Build strategy enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStrategy {
    /// Native Nix build (Linux only)
    NativeNix,
    /// Docker-based Nix build (works on all platforms)
    DockerNix,
}

impl BuildStrategy {
    /// Get a human-readable name for this strategy
    pub fn name(&self) -> &'static str {
        match self {
            BuildStrategy::NativeNix => "Native Nix",
            BuildStrategy::DockerNix => "Docker Nix",
        }
    }

    /// Get a description of this strategy
    pub fn description(&self) -> &'static str {
        match self {
            BuildStrategy::NativeNix => "Build directly using the host's Nix installation",
            BuildStrategy::DockerNix => "Build inside a Docker container with Nix",
        }
    }
}

impl std::fmt::Display for BuildStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A boxed progress callback for use with trait objects
pub type BoxedProgressCallback = Box<dyn FnMut(BuildProgress) + Send>;

/// Trait for container build strategies
///
/// Implementations of this trait handle the actual container building logic,
/// abstracting over different build methods (native Nix, Docker-based, etc.)
#[async_trait]
pub trait ContainerBuildStrategy: Send + Sync {
    /// Get the name of this strategy
    fn name(&self) -> &'static str;

    /// Get the build strategy type
    fn strategy_type(&self) -> BuildStrategy;

    /// Check if this strategy is available on the current platform
    async fn is_available(&self) -> bool;

    /// Build a container and return the path to the resulting image tarball
    ///
    /// # Arguments
    /// * `container` - Information about the container to build
    /// * `config` - Build configuration
    ///
    /// # Returns
    /// Path to the built container image (OCI tarball)
    async fn build(&self, container: &ContainerInfo, config: &BuildConfig) -> Result<PathBuf>;

    /// Build a container with progress reporting
    ///
    /// # Arguments
    /// * `container` - Information about the container to build
    /// * `config` - Build configuration
    /// * `progress` - Boxed callback for progress updates
    ///
    /// # Returns
    /// Path to the built container image (OCI tarball)
    async fn build_with_progress(
        &self,
        container: &ContainerInfo,
        config: &BuildConfig,
        progress: BoxedProgressCallback,
    ) -> Result<PathBuf>;
}

/// Create a build strategy based on platform information and configuration
pub fn create_strategy(
    platform: &PlatformInfo,
    config: &BuildConfig,
) -> Box<dyn ContainerBuildStrategy> {
    create_strategy_with_workspace(platform, config, None)
}

/// Create a build strategy with an explicit workspace root
///
/// This is used for portable mode where the workspace is extracted from embedded files.
/// When `workspace_root` is `Some`, the Docker strategy will use that path instead of
/// trying to detect the repository root via `.git` directory traversal.
pub fn create_strategy_with_workspace(
    platform: &PlatformInfo,
    config: &BuildConfig,
    workspace_root: Option<PathBuf>,
) -> Box<dyn ContainerBuildStrategy> {
    // Determine which strategy to use
    let use_docker = config.force_docker
        || (!config.force_native && (platform.platform.is_darwin() || platform.in_container));

    if use_docker {
        let volume = config
            .nix_store_volume
            .clone()
            .unwrap_or_else(|| platform.default_nix_store_volume());

        if let Some(root) = workspace_root {
            Box::new(DockerNixStrategy::with_workspace_root(
                platform.clone(),
                volume,
                root,
            ))
        } else {
            Box::new(DockerNixStrategy::new(platform.clone(), volume))
        }
    } else {
        Box::new(NativeNixStrategy::new(platform.clone()))
    }
}

/// Helper function to find the repository root
pub(crate) fn find_repo_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    // Walk up the directory tree looking for .git
    let mut dir = current_dir.as_path();
    loop {
        if dir.join(".git").exists() {
            return Ok(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => {
                // Fallback to current directory
                return Ok(current_dir);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_name() {
        assert_eq!(BuildStrategy::NativeNix.name(), "Native Nix");
        assert_eq!(BuildStrategy::DockerNix.name(), "Docker Nix");
    }

    #[test]
    fn test_strategy_display() {
        assert_eq!(BuildStrategy::NativeNix.to_string(), "Native Nix");
        assert_eq!(BuildStrategy::DockerNix.to_string(), "Docker Nix");
    }
}

//! Docker-based Nix build strategy
//!
//! This strategy builds containers by running Nix inside a Docker container.
//! It works on all platforms (including macOS) and in container environments.

use crate::config::BuildConfig;
use crate::discovery::ContainerInfo;
use crate::error::{NixContainerError, Result};
use crate::platform::PlatformInfo;
use crate::progress::{BuildPhase, BuildProgress};
use crate::strategy::{find_repo_root, BoxedProgressCallback, BuildStrategy, ContainerBuildStrategy};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Docker-based Nix build strategy
///
/// Builds containers by running Nix inside a Docker container.
/// This is the cross-platform solution that works on macOS and inside containers.
pub struct DockerNixStrategy {
    platform: PlatformInfo,
    nix_store_volume: String,
    /// Path to the workspace root (either extracted embedded workspace or repo root)
    workspace_root: Option<PathBuf>,
}

impl DockerNixStrategy {
    /// Create a new Docker-based Nix strategy
    pub fn new(platform: PlatformInfo, nix_store_volume: String) -> Self {
        Self {
            platform,
            nix_store_volume,
            workspace_root: None,
        }
    }

    /// Create a new Docker-based Nix strategy with an explicit workspace root
    ///
    /// This is used for portable mode where the workspace is extracted from embedded files.
    pub fn with_workspace_root(
        platform: PlatformInfo,
        nix_store_volume: String,
        workspace_root: PathBuf,
    ) -> Self {
        Self {
            platform,
            nix_store_volume,
            workspace_root: Some(workspace_root),
        }
    }

    /// Get the workspace root path
    ///
    /// If a workspace root was explicitly set, use that.
    /// Otherwise, fall back to finding the repo root via .git detection.
    fn get_repo_root(&self) -> Result<PathBuf> {
        if let Some(ref root) = self.workspace_root {
            Ok(root.clone())
        } else {
            find_repo_root()
        }
    }

    /// Calculate the relative path from repo root to container directory
    fn get_relative_container_path(&self, container_path: &PathBuf) -> Result<String> {
        let repo_root = self.get_repo_root()?;
        let relative = container_path
            .strip_prefix(&repo_root)
            .map_err(|_| NixContainerError::Other(format!(
                "Container path {} is not under repo root {}",
                container_path.display(),
                repo_root.display()
            )))?;
        Ok(relative.display().to_string())
    }

    /// Run the Docker-based Nix build
    async fn run_docker_nix_build(
        &self,
        container: &ContainerInfo,
        config: &BuildConfig,
    ) -> Result<PathBuf> {
        let repo_root = self.get_repo_root()?;
        let relative_path = self.get_relative_container_path(&container.path)?;

        info!(
            "Building with Docker Nix: {}/{}#{}",
            repo_root.display(),
            relative_path,
            config.nix_attribute
        );

        // Create a temporary file for the output
        let temp_dir = tempfile::tempdir()?;
        let output_path = temp_dir.path().join("image.tar.gz");

        // Build the Docker command
        let nix_script = format!(
            r#"
set -e

# Enable flakes
echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# Debug: Show current directory and contents
echo "Working directory: $(pwd)"
echo "Contents:"
ls -la

# Verify flake.nix exists before building
if [ ! -f flake.nix ]; then
    echo "ERROR: flake.nix not found in $(pwd)" >&2
    echo "Available files:" >&2
    find . -name "flake.nix" 2>/dev/null | head -20 >&2
    exit 1
fi

echo "Building {}#{}..."

# Run nix build and capture output separately
if ! nix build .#{} --no-link --print-out-paths > /tmp/nix-output.txt 2>&1; then
    echo "ERROR: Nix build failed:" >&2
    cat /tmp/nix-output.txt >&2
    exit 1
fi

# Get the image path from successful build
image_path=$(tail -1 /tmp/nix-output.txt)

if [ -z "$image_path" ]; then
    echo "ERROR: No output path from nix build" >&2
    cat /tmp/nix-output.txt >&2
    exit 1
fi

if [ ! -f "$image_path" ]; then
    echo "ERROR: Built image not found at: $image_path" >&2
    echo "Nix output was:" >&2
    cat /tmp/nix-output.txt >&2
    exit 1
fi

echo "Build complete: $image_path"
cat "$image_path"
"#,
            relative_path, config.nix_attribute, config.nix_attribute
        );

        let mut cmd = Command::new("docker");
        cmd.args([
            "run",
            "--rm",
            "-v",
            &format!("{}:/workspace", repo_root.display()),
            "--mount",
            &format!("type=volume,source={},target=/nix", self.nix_store_volume),
            "-w",
            &format!("/workspace/{}", relative_path),
            "--platform",
            self.platform.docker_platform(),
            "nixos/nix:latest",
            "sh",
            "-c",
            &nix_script,
        ]);

        debug!("Running Docker command: {:?}", cmd);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Docker Nix build failed: {}", stderr);
            return Err(NixContainerError::BuildFailed {
                container: container.name.clone(),
                message: stderr.to_string(),
            });
        }

        // The stdout contains the tarball data
        let tarball_data = output.stdout;

        if tarball_data.is_empty() {
            return Err(NixContainerError::BuildFailed {
                container: container.name.clone(),
                message: "Docker Nix build produced no output".to_string(),
            });
        }

        // Write the tarball to the temp file
        std::fs::write(&output_path, &tarball_data)?;

        debug!("Wrote {} bytes to {}", tarball_data.len(), output_path.display());

        // We need to keep the temp dir alive, so we "leak" it into a permanent path
        // by copying to a new location
        let permanent_path = std::env::temp_dir().join(format!(
            "nix-container-{}-{}.tar.gz",
            container.name,
            std::process::id()
        ));
        std::fs::copy(&output_path, &permanent_path)?;

        Ok(permanent_path)
    }
}

#[async_trait]
impl ContainerBuildStrategy for DockerNixStrategy {
    fn name(&self) -> &'static str {
        "Docker Nix"
    }

    fn strategy_type(&self) -> BuildStrategy {
        BuildStrategy::DockerNix
    }

    async fn is_available(&self) -> bool {
        self.platform.can_build_docker()
    }

    async fn build(&self, container: &ContainerInfo, config: &BuildConfig) -> Result<PathBuf> {
        self.run_docker_nix_build(container, config).await
    }

    async fn build_with_progress(
        &self,
        container: &ContainerInfo,
        config: &BuildConfig,
        mut progress: BoxedProgressCallback,
    ) -> Result<PathBuf> {
        // Report resolving phase
        progress(BuildProgress::new(
            &container.name,
            BuildPhase::Resolving,
            format!("Preparing Docker build with volume {}...", self.nix_store_volume),
        ));

        // Report building phase
        progress(BuildProgress::new(
            &container.name,
            BuildPhase::Building { current: 1, total: 1 },
            format!(
                "Building in Docker container (platform: {})...",
                self.platform.docker_platform()
            ),
        ));

        // Run the actual build
        let result = self.run_docker_nix_build(container, config).await;

        // Report completion or failure
        match &result {
            Ok(path) => {
                progress(BuildProgress::complete(
                    &container.name,
                    format!("Built: {}", path.display()),
                ));
            }
            Err(e) => {
                progress(BuildProgress::failed(&container.name, e.to_string()));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_strategy_name() {
        let platform = PlatformInfo {
            platform: crate::platform::Platform::Darwin,
            arch: crate::platform::Architecture::Aarch64,
            nix_available: false,
            docker_available: true,
            in_container: false,
        };
        let strategy = DockerNixStrategy::new(platform, "test-volume".to_string());
        assert_eq!(strategy.name(), "Docker Nix");
        assert_eq!(strategy.strategy_type(), BuildStrategy::DockerNix);
    }
}

//! Docker-based Nix build strategy
//!
//! This strategy builds containers by running Nix inside a Docker container.
//! It works on all platforms (including macOS) and in container environments.
//!
//! Supports two build modes:
//! - **SubdirFlake**: Build from a container's own flake.nix (legacy per-container flakes)
//! - **RootFlake**: Build from the repo root flake with `nix build .#<package>` (matches container-images.sh)

use crate::config::BuildConfig;
use crate::discovery::ContainerInfo;
use crate::error::{NixContainerError, Result};
use crate::platform::PlatformInfo;
use crate::progress::{BuildPhase, BuildProgress};
use crate::strategy::{find_repo_root, BoxedProgressCallback, BuildMode, BuildStrategy, ContainerBuildStrategy};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
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

    /// Build from the repo root flake using `nix build .#<package_name>`.
    ///
    /// This matches the behavior of `bin/build/container-images.sh`:
    /// - Mounts repo at its original path (read-only)
    /// - Mounts _build/ separately for writable output
    /// - Mounts Docker socket for `docker load`
    /// - Uses `cp -L` to dereference Nix symlinks
    /// - Adds `git config --global --add safe.directory`
    /// - Passes `--no-update-lock-file` to prevent flake.lock modifications
    async fn run_root_flake_build(
        &self,
        package_name: &str,
        _config: &BuildConfig,
    ) -> Result<PathBuf> {
        let repo_root = self.get_repo_root()?;

        info!(
            "Building with Docker Nix (root flake): .#{}",
            package_name
        );

        // Create output directory under repo (matches container-images.sh _build/ pattern)
        let build_output_dir = repo_root.join("_build").join(package_name);
        std::fs::create_dir_all(&build_output_dir)?;

        let container_output_dir = format!("/build/{}", package_name);

        // Build the nix script (matches container-images.sh inner script)
        let nix_script = format!(
            r#"set -euo pipefail

echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
git config --global --add safe.directory "{repo_root}" 2>/dev/null || true

echo ">>> Building {pkg}..."
nix build ".#{pkg}" -o /tmp/result -L --no-update-lock-file

# Dereference symlink (-L) to copy actual file, not symlink
cp -L /tmp/result "{out_dir}/{pkg}.tar.gz"

# Verify file exists and has content
if [ ! -s "{out_dir}/{pkg}.tar.gz" ]; then
    echo "ERROR: Output file empty or missing" >&2
    exit 1
fi

echo ">>> Build successful: {pkg}"
"#,
            repo_root = repo_root.display(),
            pkg = package_name,
            out_dir = container_output_dir,
        );

        let mut docker_args: Vec<String> = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--platform".to_string(),
            self.platform.docker_platform().to_string(),
            // Mount repo at its original path (read-only)
            "-v".to_string(),
            format!("{}:{}:ro", repo_root.display(), repo_root.display()),
            // Mount build output directory separately (writable)
            "-v".to_string(),
            format!("{}:/build", repo_root.join("_build").display()),
            // Mount Docker socket
            "-v".to_string(),
            "/var/run/docker.sock:/var/run/docker.sock".to_string(),
            // Persistent Nix store volume
            "--mount".to_string(),
            format!("type=volume,source={},target=/nix", self.nix_store_volume),
            // Working directory is repo root
            "-w".to_string(),
            repo_root.display().to_string(),
        ];

        // Add git worktree mounts if needed
        let worktree_mounts = self.resolve_git_worktree_mounts(&repo_root)?;
        docker_args.extend(worktree_mounts);

        // Add resource limits
        if let Some(limits) = Self::detect_docker_resource_limits().await {
            docker_args.push("--cpus".to_string());
            docker_args.push(limits.cpus.to_string());
            docker_args.push("--memory".to_string());
            docker_args.push(limits.memory.clone());
        }

        // Image and command
        docker_args.push("nixos/nix:latest".to_string());
        docker_args.push("sh".to_string());
        docker_args.push("-c".to_string());
        docker_args.push(nix_script);

        let mut cmd = Command::new("docker");
        cmd.args(&docker_args);

        debug!("Running Docker root flake build: docker {}", docker_args.join(" "));

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            warn!("Docker root flake build failed: {}", stderr);
            return Err(NixContainerError::BuildFailed {
                container: package_name.to_string(),
                message: format!("{}\n{}", stdout, stderr),
            });
        }

        // Verify the output file on host
        let tarball_path = build_output_dir.join(format!("{}.tar.gz", package_name));
        if !tarball_path.exists() || std::fs::metadata(&tarball_path).map(|m| m.len()).unwrap_or(0) == 0 {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(NixContainerError::BuildFailed {
                container: package_name.to_string(),
                message: format!(
                    "Build reported success but output file missing or empty: {}\nBuild output: {}",
                    tarball_path.display(),
                    stdout,
                ),
            });
        }

        info!("Root flake build successful: {}", tarball_path.display());
        Ok(tarball_path)
    }

    /// Detect git worktree and return Docker volume mount args if needed.
    ///
    /// Mirrors the logic from bin/build/_common.sh resolve_git_mounts().
    /// When .git is a file (worktree), it contains `gitdir: /path/to/.git/worktrees/name`.
    /// We need to mount both the worktree gitdir and the main .git directory
    /// so that absolute paths in the gitdir pointer resolve correctly inside the container.
    fn resolve_git_worktree_mounts(&self, repo_root: &Path) -> Result<Vec<String>> {
        let git_path = repo_root.join(".git");

        if !git_path.is_file() {
            // Regular repo (not a worktree) — no extra mounts needed
            return Ok(vec![]);
        }

        // Parse gitdir from .git file: "gitdir: /path/to/main/.git/worktrees/name"
        let content = std::fs::read_to_string(&git_path)
            .map_err(|e| NixContainerError::Other(format!("Failed to read .git file: {}", e)))?;
        let gitdir_str = content
            .strip_prefix("gitdir: ")
            .map(|s| s.trim())
            .ok_or_else(|| NixContainerError::Other("Invalid .git file format".to_string()))?;

        // Make absolute if relative
        let gitdir = if gitdir_str.starts_with('/') {
            PathBuf::from(gitdir_str)
        } else {
            repo_root.join(gitdir_str)
        };
        let gitdir = gitdir.canonicalize().unwrap_or(gitdir);

        // Read commondir to find main .git directory
        let commondir_path = gitdir.join("commondir");
        let main_git_dir = if commondir_path.exists() {
            let commondir_content = std::fs::read_to_string(&commondir_path)
                .map_err(|e| NixContainerError::Other(format!("Failed to read commondir: {}", e)))?;
            let main = gitdir.join(commondir_content.trim());
            main.canonicalize().unwrap_or(main)
        } else {
            // Fallback: worktrees dir is inside main .git
            // /path/to/.git/worktrees/name -> /path/to/.git
            gitdir.parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| gitdir.clone())
        };

        info!(
            "Git worktree detected - mounting gitdir {} and main git {}",
            gitdir.display(),
            main_git_dir.display()
        );

        Ok(vec![
            "-v".to_string(),
            format!("{}:{}:ro", gitdir.display(), gitdir.display()),
            "-v".to_string(),
            format!("{}:{}:ro", main_git_dir.display(), main_git_dir.display()),
        ])
    }

    /// Detect Docker resource limits (CPUs, memory).
    /// Mirrors bin/build/_common.sh _detect_docker_cpus() and _detect_docker_memory().
    async fn detect_docker_resource_limits() -> Option<DockerResourceLimits> {
        let cpu_output = Command::new("docker")
            .args(["info", "--format", "{{.NCPU}}"])
            .output()
            .await
            .ok()?;
        let cpus: u32 = String::from_utf8_lossy(&cpu_output.stdout)
            .trim()
            .parse()
            .unwrap_or(4);

        let mem_output = Command::new("docker")
            .args(["info", "--format", "{{.MemTotal}}"])
            .output()
            .await
            .ok()?;
        let mem_bytes: u64 = String::from_utf8_lossy(&mem_output.stdout)
            .trim()
            .parse()
            .unwrap_or(0);

        let mem_gb = if mem_bytes > 0 {
            let gb = mem_bytes / 1_073_741_824;
            if gb > 1 { gb - 1 } else { 1 } // Reserve 1GB overhead
        } else {
            8
        };

        Some(DockerResourceLimits {
            cpus,
            memory: format!("{}g", mem_gb),
        })
    }
}

/// Docker resource limits detected from the daemon.
struct DockerResourceLimits {
    cpus: u32,
    memory: String,
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

    async fn build_with_mode(
        &self,
        mode: &BuildMode,
        config: &BuildConfig,
    ) -> Result<PathBuf> {
        match mode {
            BuildMode::RootFlake { package_name } => {
                self.run_root_flake_build(package_name, config).await
            }
            BuildMode::SubdirFlake { container_dir, attribute } => {
                let container = ContainerInfo::new(
                    container_dir.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    container_dir,
                );
                let config = BuildConfig {
                    nix_attribute: attribute.clone(),
                    ..config.clone()
                };
                self.run_docker_nix_build(&container, &config).await
            }
        }
    }

    async fn build_with_mode_and_progress(
        &self,
        mode: &BuildMode,
        config: &BuildConfig,
        mut progress: BoxedProgressCallback,
    ) -> Result<PathBuf> {
        let label = match mode {
            BuildMode::RootFlake { package_name } => package_name.clone(),
            BuildMode::SubdirFlake { container_dir, .. } => container_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        };

        progress(BuildProgress::new(
            &label,
            BuildPhase::Resolving,
            format!("Preparing Docker build with volume {}...", self.nix_store_volume),
        ));

        progress(BuildProgress::new(
            &label,
            BuildPhase::Building { current: 1, total: 1 },
            format!("Building in Docker container (platform: {})...", self.platform.docker_platform()),
        ));

        let result = self.build_with_mode(mode, config).await;

        match &result {
            Ok(path) => progress(BuildProgress::complete(&label, format!("Built: {}", path.display()))),
            Err(e) => progress(BuildProgress::failed(&label, e.to_string())),
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

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
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
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
    ///
    /// Uses tar streaming to pass the workspace to Docker, avoiding bind mount
    /// issues on macOS where /var/folders is not in Docker Desktop's file sharing.
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

        // Build the nix script that will run inside the container
        // The script first extracts the workspace from stdin, then runs nix build
        let nix_script = format!(
            r#"
set -e

# Extract workspace from stdin (tar streaming)
mkdir -p /workspace
cd /workspace
tar -xf -

# Mark workspace as safe for Git (files are owned by host user, not container user)
git config --global --add safe.directory /workspace

# Enable flakes
echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# Change to container directory
cd /workspace/{relative_path}

# Debug: Show current directory and contents
echo "Working directory: $(pwd)" >&2
echo "Contents:" >&2
ls -la >&2

# Verify flake.nix exists before building
if [ ! -f flake.nix ]; then
    echo "ERROR: flake.nix not found in $(pwd)" >&2
    exit 1
fi

echo "Building {relative_path}#{nix_attr}..." >&2

# Run nix build and capture output separately
if ! nix build .#{nix_attr} --no-link --print-out-paths > /tmp/nix-output.txt 2>&1; then
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

echo "Build complete: $image_path" >&2
cat "$image_path"
"#,
            relative_path = relative_path,
            nix_attr = config.nix_attribute
        );

        // Create tar archive of workspace
        info!("Creating tar archive of workspace...");
        let tar_output = Command::new("tar")
            .args(["-C", repo_root.to_str().unwrap(), "-cf", "-", "."])
            .output()
            .await?;

        if !tar_output.status.success() {
            let stderr = String::from_utf8_lossy(&tar_output.stderr);
            return Err(NixContainerError::Other(format!(
                "Failed to create tar archive of workspace: {}",
                stderr
            )));
        }

        info!(
            "Tar archive created ({} bytes), streaming to Docker...",
            tar_output.stdout.len()
        );

        // Run docker with tar piped to stdin (no bind mount needed)
        let mut docker_cmd = Command::new("docker");
        docker_cmd.args([
            "run",
            "-i", // Keep stdin open for tar input
            "--rm",
            "--mount",
            &format!("type=volume,source={},target=/nix", self.nix_store_volume),
            "--platform",
            self.platform.docker_platform(),
            "nixos/nix:latest",
            "sh",
            "-c",
            &nix_script,
        ]);

        docker_cmd.stdin(Stdio::piped());
        docker_cmd.stdout(Stdio::piped());
        docker_cmd.stderr(Stdio::piped());

        debug!("Running Docker command: {:?}", docker_cmd);

        let mut child = docker_cmd.spawn()?;

        // Write tar archive to docker's stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&tar_output.stdout).await?;
            // Drop stdin to signal EOF - this happens automatically when stdin goes out of scope
        }

        let output = child.wait_with_output().await?;

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

        // Write to permanent location
        let permanent_path = std::env::temp_dir().join(format!(
            "nix-container-{}-{}.tar.gz",
            container.name,
            std::process::id()
        ));
        std::fs::write(&permanent_path, &tarball_data)?;

        debug!(
            "Wrote {} bytes to {}",
            tarball_data.len(),
            permanent_path.display()
        );
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

        // Collect safe.directory paths (repo root + any path inputs from flake.lock)
        let mut safe_dirs = vec![repo_root.display().to_string()];
        let lock_path = repo_root.join("flake.lock");
        if lock_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&lock_path) {
                if let Ok(lock) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(nodes) = lock.get("nodes").and_then(|n| n.as_object()) {
                        for (_name, node) in nodes {
                            let is_path = node.get("locked")
                                .and_then(|l| l.get("type"))
                                .and_then(|t| t.as_str()) == Some("path");
                            if is_path {
                                if let Some(p) = node.get("locked").and_then(|l| l.get("path")).and_then(|p| p.as_str()) {
                                    let abs = if p.starts_with('/') {
                                        PathBuf::from(p)
                                    } else {
                                        repo_root.join(p).canonicalize().unwrap_or_else(|_| repo_root.join(p))
                                    };
                                    if abs.exists() && abs != repo_root {
                                        safe_dirs.push(abs.display().to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let safe_dir_cmds: String = safe_dirs.iter()
            .map(|d| format!(r#"git config --global --add safe.directory "{}" 2>/dev/null || true"#, d))
            .collect::<Vec<_>>()
            .join("\n");

        // Build the nix script (matches container-images.sh inner script)
        let nix_script = format!(
            r#"set -euo pipefail

echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
{safe_dir_cmds}

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
            safe_dir_cmds = safe_dir_cmds,
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

        // Add path input mounts from flake.lock (local dev support)
        let path_input_mounts = self.resolve_path_input_mounts(&repo_root)?;
        docker_args.extend(path_input_mounts);

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

    /// Parse flake.lock for `path:` inputs and return Docker volume mount args.
    ///
    /// When a consumer flake has `firestream.url = "path:../Firestream"`, the
    /// flake.lock records the resolved path. We need to mount these paths into
    /// the Docker container so that `nix build` can resolve them.
    fn resolve_path_input_mounts(&self, repo_root: &Path) -> Result<Vec<String>> {
        let lock_path = repo_root.join("flake.lock");
        if !lock_path.exists() {
            debug!("No flake.lock found at {}, skipping path input discovery", lock_path.display());
            return Ok(vec![]);
        }

        let lock_content: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&lock_path)
                .map_err(|e| NixContainerError::Other(format!("Failed to read flake.lock: {}", e)))?
        ).map_err(|e| NixContainerError::Other(format!("Failed to parse flake.lock: {}", e)))?;

        let mut mounts = vec![];
        let mut safe_dirs = vec![];

        let nodes = match lock_content.get("nodes").and_then(|n| n.as_object()) {
            Some(nodes) => nodes,
            None => return Ok(vec![]),
        };

        for (name, node) in nodes {
            // Check locked.type == "path" (resolved inputs)
            let is_path = node.get("locked")
                .and_then(|l| l.get("type"))
                .and_then(|t| t.as_str()) == Some("path");

            // Also check original.type == "path" (some lock formats)
            let is_original_path = !is_path && node.get("original")
                .and_then(|o| o.get("type"))
                .and_then(|t| t.as_str()) == Some("path");

            if !is_path && !is_original_path {
                continue;
            }

            // Get the path from locked (preferred) or original
            let path_str = node.get("locked")
                .and_then(|l| l.get("path"))
                .and_then(|p| p.as_str())
                .or_else(|| node.get("original")
                    .and_then(|o| o.get("path"))
                    .and_then(|p| p.as_str()));

            let path_str = match path_str {
                Some(p) => p,
                None => continue,
            };

            // Resolve to absolute path
            let path = if path_str.starts_with('/') {
                PathBuf::from(path_str)
            } else {
                match repo_root.join(path_str).canonicalize() {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Cannot resolve path input '{}' ({}): {}", name, path_str, e);
                        continue;
                    }
                }
            };

            // Skip if same as repo root (already mounted) or doesn't exist
            if path == repo_root {
                continue;
            }
            if !path.exists() {
                warn!("Path input '{}' points to non-existent path: {}", name, path.display());
                continue;
            }

            info!("Mounting path input '{}': {}", name, path.display());
            mounts.push("-v".to_string());
            mounts.push(format!("{}:{}:ro", path.display(), path.display()));
            safe_dirs.push(path.display().to_string());

            // Also resolve git worktree mounts for this path input
            if let Ok(wt_mounts) = self.resolve_git_worktree_mounts(&path) {
                mounts.extend(wt_mounts);
            }
        }

        // Store safe_dirs for use in the nix script (attached to self via the mounts)
        // We embed them as environment variable mounts
        if !safe_dirs.is_empty() {
            debug!("Path input safe directories: {:?}", safe_dirs);
        }

        Ok(mounts)
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
    use std::io::Write;

    fn make_strategy() -> DockerNixStrategy {
        let platform = PlatformInfo {
            platform: crate::platform::Platform::Darwin,
            arch: crate::platform::Architecture::Aarch64,
            nix_available: false,
            docker_available: true,
            in_container: false,
        };
        DockerNixStrategy::new(platform, "test-volume".to_string())
    }

    #[test]
    fn test_docker_strategy_name() {
        let strategy = make_strategy();
        assert_eq!(strategy.name(), "Docker Nix");
        assert_eq!(strategy.strategy_type(), BuildStrategy::DockerNix);
    }

    #[test]
    fn test_resolve_path_input_mounts_no_lockfile() {
        let strategy = make_strategy();
        let tmp = tempfile::tempdir().unwrap();
        // No flake.lock => empty result
        let mounts = strategy.resolve_path_input_mounts(tmp.path()).unwrap();
        assert!(mounts.is_empty());
    }

    #[test]
    fn test_resolve_path_input_mounts_no_path_inputs() {
        let strategy = make_strategy();
        let tmp = tempfile::tempdir().unwrap();
        let lock = serde_json::json!({
            "nodes": {
                "nixpkgs": {
                    "locked": {
                        "type": "github",
                        "owner": "NixOS",
                        "repo": "nixpkgs",
                        "rev": "abc123"
                    }
                },
                "root": {}
            },
            "root": "root",
            "version": 7
        });
        let mut f = std::fs::File::create(tmp.path().join("flake.lock")).unwrap();
        f.write_all(serde_json::to_string(&lock).unwrap().as_bytes()).unwrap();

        let mounts = strategy.resolve_path_input_mounts(tmp.path()).unwrap();
        assert!(mounts.is_empty());
    }

    #[test]
    fn test_resolve_path_input_mounts_with_path_input() {
        let strategy = make_strategy();
        let tmp = tempfile::tempdir().unwrap();
        // Create a directory to act as the path input target
        let input_dir = tempfile::tempdir().unwrap();
        let input_path = input_dir.path().to_str().unwrap().to_string();

        let lock = serde_json::json!({
            "nodes": {
                "firestream": {
                    "locked": {
                        "type": "path",
                        "path": input_path
                    }
                },
                "root": {}
            },
            "root": "root",
            "version": 7
        });
        let mut f = std::fs::File::create(tmp.path().join("flake.lock")).unwrap();
        f.write_all(serde_json::to_string(&lock).unwrap().as_bytes()).unwrap();

        let mounts = strategy.resolve_path_input_mounts(tmp.path()).unwrap();
        // Should have at least -v and the mount spec
        assert!(mounts.len() >= 2, "Expected mount args, got: {:?}", mounts);
        assert_eq!(mounts[0], "-v");
        assert!(mounts[1].contains(&input_path));
        assert!(mounts[1].ends_with(":ro"));
    }

    #[test]
    fn test_resolve_path_input_mounts_skips_nonexistent() {
        let strategy = make_strategy();
        let tmp = tempfile::tempdir().unwrap();

        let lock = serde_json::json!({
            "nodes": {
                "missing": {
                    "locked": {
                        "type": "path",
                        "path": "/nonexistent/path/that/should/not/exist"
                    }
                },
                "root": {}
            },
            "root": "root",
            "version": 7
        });
        let mut f = std::fs::File::create(tmp.path().join("flake.lock")).unwrap();
        f.write_all(serde_json::to_string(&lock).unwrap().as_bytes()).unwrap();

        let mounts = strategy.resolve_path_input_mounts(tmp.path()).unwrap();
        assert!(mounts.is_empty(), "Should skip non-existent paths");
    }
}

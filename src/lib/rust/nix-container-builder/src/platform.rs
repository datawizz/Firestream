//! Platform and architecture detection

use crate::error::{NixContainerError, Result};
use crate::strategy::BuildStrategy;
use std::path::Path;
use tokio::process::Command;

/// Operating system platform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Darwin,
}

impl Platform {
    /// Detect the current platform
    pub fn detect() -> Result<Self> {
        match std::env::consts::OS {
            "linux" => Ok(Platform::Linux),
            "macos" => Ok(Platform::Darwin),
            other => Err(NixContainerError::UnsupportedPlatform(other.to_string())),
        }
    }

    /// Check if this is Linux
    pub fn is_linux(&self) -> bool {
        matches!(self, Platform::Linux)
    }

    /// Check if this is macOS/Darwin
    pub fn is_darwin(&self) -> bool {
        matches!(self, Platform::Darwin)
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Linux => write!(f, "Linux"),
            Platform::Darwin => write!(f, "macOS"),
        }
    }
}

/// CPU architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
}

impl Architecture {
    /// Detect the current architecture
    pub fn detect() -> Result<Self> {
        match std::env::consts::ARCH {
            "x86_64" => Ok(Architecture::X86_64),
            "aarch64" => Ok(Architecture::Aarch64),
            other => Err(NixContainerError::UnsupportedPlatform(format!(
                "Unsupported architecture: {}",
                other
            ))),
        }
    }

    /// Get the Docker platform string for this architecture
    pub fn docker_platform(&self) -> &'static str {
        match self {
            Architecture::X86_64 => "linux/amd64",
            Architecture::Aarch64 => "linux/arm64",
        }
    }

    /// Get the Nix store volume suffix for this architecture
    pub fn volume_suffix(&self) -> &'static str {
        match self {
            Architecture::X86_64 => "amd64",
            Architecture::Aarch64 => "arm64",
        }
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::Aarch64 => write!(f, "aarch64"),
        }
    }
}

/// Complete platform information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    /// Operating system platform
    pub platform: Platform,

    /// CPU architecture
    pub arch: Architecture,

    /// Whether Nix is available in PATH
    pub nix_available: bool,

    /// Whether Docker is available and accessible
    pub docker_available: bool,

    /// Whether we're running inside a container
    pub in_container: bool,
}

impl PlatformInfo {
    /// Detect all platform information
    pub async fn detect() -> Result<Self> {
        let platform = Platform::detect()?;
        let arch = Architecture::detect()?;

        let nix_available = check_nix_available().await;
        let docker_available = check_docker_available().await;
        let in_container = is_running_in_container();

        Ok(Self {
            platform,
            arch,
            nix_available,
            docker_available,
            in_container,
        })
    }

    /// Get the recommended build strategy for this platform
    pub fn recommended_strategy(&self) -> BuildStrategy {
        // On Linux with native Nix, prefer native builds
        if self.platform.is_linux() && self.nix_available && !self.in_container {
            BuildStrategy::NativeNix
        }
        // On macOS or in containers, use Docker-based builds
        else if self.docker_available {
            BuildStrategy::DockerNix
        }
        // Fallback to native if available
        else if self.nix_available {
            BuildStrategy::NativeNix
        }
        // Default to Docker
        else {
            BuildStrategy::DockerNix
        }
    }

    /// Get the Docker platform string
    pub fn docker_platform(&self) -> &'static str {
        self.arch.docker_platform()
    }

    /// Get the default Nix store volume name
    pub fn default_nix_store_volume(&self) -> String {
        format!("firestream-nix-store-{}", self.arch.volume_suffix())
    }

    /// Check if native Nix builds are possible
    pub fn can_build_native(&self) -> bool {
        self.platform.is_linux() && self.nix_available && !self.in_container
    }

    /// Check if Docker-based builds are possible
    pub fn can_build_docker(&self) -> bool {
        self.docker_available
    }
}

/// Check if Nix is available in PATH
async fn check_nix_available() -> bool {
    which::which("nix").is_ok()
}

/// Check if Docker is available and the daemon is accessible
async fn check_docker_available() -> bool {
    if which::which("docker").is_err() {
        return false;
    }

    // Try to ping Docker daemon
    let output = Command::new("docker")
        .args(["info"])
        .output()
        .await;

    matches!(output, Ok(o) if o.status.success())
}

/// Check if we're running inside a container
fn is_running_in_container() -> bool {
    // Check for .dockerenv file
    if Path::new("/.dockerenv").exists() {
        return true;
    }

    // Check for Kubernetes pod
    if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        return true;
    }

    // Check common container environment variables
    if std::env::var("CONTAINER").is_ok()
        || std::env::var("container").is_ok()
        || std::env::var("CONTAINER_REGISTRY_URL").is_ok()
    {
        return true;
    }

    // Check cgroups for container indicators
    if let Ok(cgroups) = std::fs::read_to_string("/proc/1/cgroup") {
        if cgroups.contains("docker") || cgroups.contains("kubepods") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        assert!(platform.is_ok());
    }

    #[test]
    fn test_architecture_detection() {
        let arch = Architecture::detect();
        assert!(arch.is_ok());
    }

    #[test]
    fn test_docker_platform_string() {
        assert_eq!(Architecture::X86_64.docker_platform(), "linux/amd64");
        assert_eq!(Architecture::Aarch64.docker_platform(), "linux/arm64");
    }

    #[test]
    fn test_volume_suffix() {
        assert_eq!(Architecture::X86_64.volume_suffix(), "amd64");
        assert_eq!(Architecture::Aarch64.volume_suffix(), "arm64");
    }
}

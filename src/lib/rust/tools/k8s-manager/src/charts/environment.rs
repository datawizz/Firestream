//! Environment detection and configuration
//!
//! This module handles detecting the system environment including
//! CPU architecture, memory, Docker availability, and GPU presence.

use crate::core::{FirestreamError, Result};
use serde::{Deserialize, Serialize};
use std::env;
use sysinfo::System;
use tracing::{info, warn};

/// System environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub machine_id: String,
    pub cpu_architecture: String,
    pub total_cpu_cores: usize,
    pub total_memory_mb: u64,
    pub has_nvidia_gpu: bool,
    pub has_docker: bool,
    pub docker_version: Option<String>,
    pub is_in_container: bool,
    pub deployment_mode: Option<String>,
}

impl Environment {
    /// Detect the current system environment
    pub async fn detect() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Get machine ID
        let machine_id = Self::get_machine_id();

        // Get CPU architecture
        let cpu_architecture = std::env::consts::ARCH.to_string();
        info!("CPU Architecture: {}", cpu_architecture);

        // Get CPU cores
        let total_cpu_cores = sys.cpus().len();
        info!("Total CPU cores (including Hyperthreads): {}", total_cpu_cores);

        // Get memory in MB
        let total_memory_mb = sys.total_memory() / 1024 / 1024;
        info!("Total Memory resources (in MB): {}", total_memory_mb);

        // Check for NVIDIA GPU
        let has_nvidia_gpu = Self::check_nvidia_gpu().await;
        if has_nvidia_gpu {
            info!("NVIDIA GPU detected");
        } else {
            info!("No NVIDIA GPU detected");
        }

        // Check Docker
        let (has_docker, docker_version) = Self::check_docker_availability().await;

        // Check if running in container
        let is_in_container = Self::is_running_in_container();

        // Get deployment mode from environment
        let deployment_mode = env::var("DEPLOYMENT_MODE").ok();

        Ok(Environment {
            machine_id,
            cpu_architecture,
            total_cpu_cores,
            total_memory_mb,
            has_nvidia_gpu,
            has_docker,
            docker_version,
            is_in_container,
            deployment_mode,
        })
    }

    /// Get machine ID
    fn get_machine_id() -> String {
        // Try to get from environment first
        if let Ok(machine_id) = env::var("MACHINE_ID") {
            return machine_id;
        }

        // Try to read from /var/lib/dbus/machine-id
        if let Ok(machine_id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
            return machine_id.trim().to_string();
        }

        // Try /etc/machine-id
        if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
            return machine_id.trim().to_string();
        }

        // Generate a random one if not found
        uuid::Uuid::new_v4().to_string()
    }

    /// Check for NVIDIA GPU
    async fn check_nvidia_gpu() -> bool {
        // Check if nvidia-smi is available
        if let Ok(output) = tokio::process::Command::new("nvidia-smi")
            .output()
            .await
        {
            return output.status.success();
        }

        // Check lspci for NVIDIA devices
        if let Ok(output) = tokio::process::Command::new("lspci")
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout.to_lowercase().contains("nvidia");
            }
        }

        false
    }

    /// Check Docker availability and version
    async fn check_docker_availability() -> (bool, Option<String>) {
        match tokio::process::Command::new("docker")
            .arg("--version")
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                info!("Docker version {} is installed", version);
                (true, Some(version))
            }
            _ => {
                warn!("Docker is not available in the terminal");
                (false, None)
            }
        }
    }

    /// Check if running inside a container
    fn is_running_in_container() -> bool {
        // Check for .dockerenv
        if std::path::Path::new("/.dockerenv").exists() {
            return true;
        }

        // Check cgroup
        if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
            if cgroup.contains("/docker/") || cgroup.contains("/docker-ce/") {
                return true;
            }
        }

        if let Ok(cgroup) = std::fs::read_to_string("/proc/self/cgroup") {
            if cgroup.contains("/docker/") || cgroup.contains("/docker-ce/") {
                return true;
            }
        }

        false
    }

    /// Validate the environment for the given requirements
    pub fn validate(&self, require_docker: bool, require_gpu: bool) -> Result<()> {
        if require_docker && !self.has_docker {
            return Err(FirestreamError::GeneralError(
                "Docker is required but not available".to_string()
            ));
        }

        if require_gpu && !self.has_nvidia_gpu {
            warn!("GPU is required but not detected. Some features may not work properly.");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_detection() {
        let env = Environment::detect().await.unwrap();
        
        // Basic sanity checks
        assert!(env.total_cpu_cores > 0);
        assert!(env.total_memory_mb > 0);
        assert!(!env.cpu_architecture.is_empty());
        assert!(!env.machine_id.is_empty());
    }
}

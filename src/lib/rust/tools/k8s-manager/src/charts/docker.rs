//! Docker integration module
//!
//! This module provides Docker-related functionality including
//! checking Docker availability and managing Docker operations.

use crate::core::{FirestreamError, Result};
use bollard::Docker;
use std::path::Path;
use tracing::info;

/// Check if Docker is available and accessible
pub async fn check_docker() -> Result<()> {
    // Check if docker command exists
    check_docker_command().await?;
    
    // Check Docker socket access
    check_docker_socket()?;
    
    // Try to connect to Docker daemon
    check_docker_daemon().await?;
    
    Ok(())
}

/// Check if docker command is available
async fn check_docker_command() -> Result<()> {
    match which::which("docker") {
        Ok(path) => {
            info!("Docker found at: {:?}", path);
            
            // Get version
            match tokio::process::Command::new("docker")
                .arg("--version")
                .output()
                .await
            {
                Ok(output) if output.status.success() => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    info!("Docker version: {}", version.trim());
                    Ok(())
                }
                _ => Err(FirestreamError::GeneralError(
                    "Failed to get Docker version".to_string()
                )),
            }
        }
        Err(_) => Err(FirestreamError::GeneralError(
            "Docker is not available in the terminal".to_string()
        )),
    }
}

/// Check Docker socket access
fn check_docker_socket() -> Result<()> {
    let socket_path = "/var/run/docker.sock";
    
    if !Path::new(socket_path).exists() {
        return Err(FirestreamError::GeneralError(
            format!("{} does not exist", socket_path)
        ));
    }
    
    // Check if we can read and write to the socket
    match std::fs::metadata(socket_path) {
        Ok(metadata) => {
            use std::os::unix::fs::PermissionsExt;
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            
            // Check if current user has read/write access
            // This is a simplified check - in reality we'd need to check group membership too
            if mode & 0o006 == 0o006 {
                info!("Docker socket is accessible at {}", socket_path);
                Ok(())
            } else {
                Err(FirestreamError::GeneralError(
                    format!("Insufficient permissions on {}", socket_path)
                ))
            }
        }
        Err(e) => Err(FirestreamError::GeneralError(
            format!("Cannot access Docker socket: {}", e)
        )),
    }
}

/// Check Docker daemon connectivity
async fn check_docker_daemon() -> Result<()> {
    match Docker::connect_with_socket_defaults() {
        Ok(docker) => {
            // Try a simple operation to verify connectivity
            match docker.version().await {
                Ok(version) => {
                    info!("Connected to Docker daemon version: {:?}", version.version);
                    Ok(())
                }
                Err(e) => Err(FirestreamError::GeneralError(
                    format!("Failed to connect to Docker daemon: {}", e)
                )),
            }
        }
        Err(e) => Err(FirestreamError::GeneralError(
            format!("Failed to create Docker client: {}", e)
        )),
    }
}

/// Get Docker client instance
pub async fn get_docker_client() -> Result<Docker> {
    Docker::connect_with_socket_defaults()
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to connect to Docker: {}", e)))
}

/// Check if we're running inside a Docker container
pub fn is_running_in_docker() -> bool {
    // Check for .dockerenv
    if Path::new("/.dockerenv").exists() {
        return true;
    }
    
    // Check cgroup
    if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
        return cgroup.contains("/docker/") || cgroup.contains("/docker-ce/");
    }
    
    false
}

/// Run a Docker Compose command
pub async fn docker_compose(args: &[&str], working_dir: Option<&str>) -> Result<()> {
    let mut cmd = tokio::process::Command::new("docker");
    cmd.arg("compose");
    
    for arg in args {
        cmd.arg(arg);
    }
    
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }
    
    let output = cmd.output().await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to run docker compose: {}", e)))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FirestreamError::GeneralError(
            format!("Docker compose failed: {}", stderr)
        ));
    }
    
    Ok(())
}

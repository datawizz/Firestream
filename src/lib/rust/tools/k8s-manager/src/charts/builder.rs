//! Container image builder
//!
//! This module handles building container images for Firestream services.

use crate::core::{FirestreamError, Result};
use crate::deploy::environment::Environment;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::info;

/// Build context information
#[derive(Debug)]
pub struct BuildContext {
    pub service_name: String,
    pub dockerfile_path: PathBuf,
    pub context_path: PathBuf,
    pub tag: String,
    pub build_args: Vec<(String, String)>,
}

/// Build container images
pub async fn build_images(env: &Environment) -> Result<()> {
    info!("Building container images");
    
    // Get Git commit hash for tagging
    let git_hash = get_git_hash().await.unwrap_or_else(|_| "latest".to_string());
    
    // Find services to build
    let services = find_services_to_build().await?;
    
    // Build each service
    for service in services {
        build_service_image(&service, &git_hash, env).await?;
    }
    
    Ok(())
}

/// Get current Git commit hash
async fn get_git_hash() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to get git hash: {}", e)))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(FirestreamError::GeneralError("Failed to get git commit hash".to_string()))
    }
}

/// Find services that need to be built
async fn find_services_to_build() -> Result<Vec<BuildContext>> {
    let services = Vec::new();
    
    // Look for Dockerfiles in the project
    // TODO: Implement proper service discovery
    
    // For now, return an empty list as we don't know the exact structure
    Ok(services)
}

/// Build a service image
async fn build_service_image(context: &BuildContext, git_hash: &str, env: &Environment) -> Result<()> {
    info!("Building image for service: {}", context.service_name);
    
    let tag = format!("{}:{}", context.tag, git_hash);
    
    let mut cmd = Command::new("docker");
    cmd.args(&["build", "-t", &tag]);
    
    // Add build arguments
    for (key, value) in &context.build_args {
        cmd.args(&["--build-arg", &format!("{}={}", key, value)]);
    }
    
    // Add platform if needed
    if env.cpu_architecture == "aarch64" || env.cpu_architecture == "arm64" {
        cmd.args(&["--platform", "linux/arm64"]);
    }
    
    // Add Dockerfile path
    cmd.args(&["-f", context.dockerfile_path.to_str().unwrap()]);
    
    // Add context path
    cmd.arg(context.context_path.to_str().unwrap());
    
    let status = cmd.status().await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to build image: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            format!("Failed to build image for service '{}'", context.service_name)
        ));
    }
    
    // Tag as latest
    tag_image(&tag, &format!("{}:latest", context.tag)).await?;
    
    Ok(())
}

/// Tag a Docker image
async fn tag_image(source: &str, target: &str) -> Result<()> {
    let status = Command::new("docker")
        .args(&["tag", source, target])
        .status()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to tag image: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            format!("Failed to tag image '{}' as '{}'", source, target)
        ));
    }
    
    Ok(())
}

/// Push image to registry
pub async fn push_image(image: &str) -> Result<()> {
    info!("Pushing image: {}", image);
    
    let status = Command::new("docker")
        .args(&["push", image])
        .status()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to push image: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            format!("Failed to push image '{}'", image)
        ));
    }
    
    Ok(())
}

/// Pull image from registry
pub async fn pull_image(image: &str) -> Result<()> {
    info!("Pulling image: {}", image);
    
    let status = Command::new("docker")
        .args(&["pull", image])
        .status()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to pull image: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            format!("Failed to pull image '{}'", image)
        ));
    }
    
    Ok(())
}

/// Clean up old images
pub async fn cleanup_images(_keep_latest: usize) -> Result<()> {
    info!("Cleaning up old Docker images");
    
    // TODO: Implement image cleanup logic
    // - List images by repository
    // - Sort by creation date
    // - Keep only the latest N images
    // - Remove the rest
    
    Ok(())
}

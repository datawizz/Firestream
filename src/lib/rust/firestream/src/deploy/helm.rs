//! Helm chart management
//!
//! This module handles Helm chart installation and management.

use crate::core::{FirestreamError, Result};
use std::path::Path;
use tokio::process::Command;
use tracing::info;

/// Helm repository information
#[derive(Debug, Clone)]
pub struct HelmRepo {
    pub name: String,
    pub url: String,
}

impl HelmRepo {
    pub fn bitnami() -> Self {
        Self {
            name: "bitnami".to_string(),
            url: "https://charts.bitnami.com/bitnami".to_string(),
        }
    }
}

/// Install Helm charts
pub async fn install_charts() -> Result<()> {
    // Check if helm is installed
    check_helm_installed().await?;
    
    // Add required repositories
    add_repositories().await?;
    
    // Update repositories
    update_repositories().await?;
    
    // Install charts
    // TODO: Load chart configurations from config files
    install_firestream_charts().await?;
    
    Ok(())
}

/// Check if helm is installed
async fn check_helm_installed() -> Result<()> {
    match which::which("helm") {
        Ok(path) => {
            info!("helm found at: {:?}", path);
            
            // Get version
            match Command::new("helm")
                .arg("version")
                .arg("--short")
                .output()
                .await
            {
                Ok(output) if output.status.success() => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    info!("helm version: {}", version.trim());
                    Ok(())
                }
                _ => Err(FirestreamError::GeneralError(
                    "Failed to get helm version".to_string()
                )),
            }
        }
        Err(_) => Err(FirestreamError::GeneralError(
            "helm is not installed. Please install helm first.".to_string()
        )),
    }
}

/// Add Helm repositories
async fn add_repositories() -> Result<()> {
    let repos = vec![
        HelmRepo::bitnami(),
        // Add more repositories as needed
    ];
    
    for repo in repos {
        add_repository(&repo).await?;
    }
    
    Ok(())
}

/// Add a single Helm repository
async fn add_repository(repo: &HelmRepo) -> Result<()> {
    info!("Adding Helm repository '{}'", repo.name);
    
    let status = Command::new("helm")
        .args(&["repo", "add", &repo.name, &repo.url, "--force-update"])
        .status()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to add helm repo: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            format!("Failed to add Helm repository '{}'", repo.name)
        ));
    }
    
    Ok(())
}

/// Update Helm repositories
async fn update_repositories() -> Result<()> {
    info!("Updating Helm repositories");
    
    let status = Command::new("helm")
        .args(&["repo", "update"])
        .status()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to update helm repos: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            "Failed to update Helm repositories".to_string()
        ));
    }
    
    Ok(())
}

/// Install Firestream charts
async fn install_firestream_charts() -> Result<()> {
    // TODO: Read chart configurations from files
    // For now, we'll just simulate the installation
    info!("Installing Firestream Helm charts");
    
    // Example: Install PostgreSQL
    // install_chart("postgresql", "bitnami/postgresql", None).await?;
    
    // Example: Install Kafka
    // install_chart("kafka", "bitnami/kafka", None).await?;
    
    Ok(())
}

/// Install a Helm chart
pub async fn install_chart(release_name: &str, chart: &str, values_file: Option<&Path>) -> Result<()> {
    info!("Installing Helm chart '{}' as '{}'", chart, release_name);
    
    let mut cmd = Command::new("helm");
    cmd.args(&["install", release_name, chart]);
    
    if let Some(values) = values_file {
        cmd.args(&["-f", values.to_str().unwrap()]);
    }
    
    // Add common flags
    cmd.args(&["--create-namespace", "--wait"]);
    
    let status = cmd.status().await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to install chart: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            format!("Failed to install Helm chart '{}'", chart)
        ));
    }
    
    Ok(())
}

/// Upgrade a Helm release
pub async fn upgrade_release(release_name: &str, chart: &str, values_file: Option<&Path>) -> Result<()> {
    info!("Upgrading Helm release '{}'", release_name);
    
    let mut cmd = Command::new("helm");
    cmd.args(&["upgrade", release_name, chart, "--install"]);
    
    if let Some(values) = values_file {
        cmd.args(&["-f", values.to_str().unwrap()]);
    }
    
    // Add common flags
    cmd.args(&["--create-namespace", "--wait"]);
    
    let status = cmd.status().await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to upgrade release: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            format!("Failed to upgrade Helm release '{}'", release_name)
        ));
    }
    
    Ok(())
}

/// Uninstall a Helm release
pub async fn uninstall_release(release_name: &str) -> Result<()> {
    info!("Uninstalling Helm release '{}'", release_name);
    
    let status = Command::new("helm")
        .args(&["uninstall", release_name])
        .status()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to uninstall release: {}", e)))?;
    
    if !status.success() {
        return Err(FirestreamError::GeneralError(
            format!("Failed to uninstall Helm release '{}'", release_name)
        ));
    }
    
    Ok(())
}

/// List Helm releases
pub async fn list_releases() -> Result<Vec<String>> {
    let output = Command::new("helm")
        .args(&["list", "-q"])
        .output()
        .await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to list releases: {}", e)))?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    } else {
        Ok(vec![])
    }
}

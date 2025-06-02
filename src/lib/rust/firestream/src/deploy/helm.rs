//! Helm chart management
//!
//! This module provides both legacy functions and the new lifecycle-based system
//! for managing Helm charts.

use crate::core::{FirestreamError, Result};
use crate::deploy::helm_lifecycle::{
    HelmChart, ChartInfo, HelmAction, CommonChart, AppLifecycle,
    charts::{PrometheusOperatorChart, ExternalDnsChart, NginxIngressChart,
             PostgresqlChart, KafkaChart},
};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{info, debug};

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
    
    pub fn prometheus_community() -> Self {
        Self {
            name: "prometheus-community".to_string(),
            url: "https://prometheus-community.github.io/helm-charts".to_string(),
        }
    }
    
    pub fn ingress_nginx() -> Self {
        Self {
            name: "ingress-nginx".to_string(),
            url: "https://kubernetes.github.io/ingress-nginx".to_string(),
        }
    }
    
    pub fn strimzi() -> Self {
        Self {
            name: "strimzi".to_string(),
            url: "https://strimzi.io/charts/".to_string(),
        }
    }
}

/// Install Helm charts using the lifecycle system
pub async fn install_charts() -> Result<()> {
    // Check if helm is installed
    check_helm_installed().await?;
    
    // Add required repositories
    add_repositories().await?;
    
    // Update repositories
    update_repositories().await?;
    
    // Install charts using the new lifecycle system
    install_firestream_charts_lifecycle().await?;
    
    Ok(())
}

/// Install Firestream charts using the lifecycle system
async fn install_firestream_charts_lifecycle() -> Result<()> {
    info!("Installing Firestream Helm charts using lifecycle management");
    
    // Get kubeconfig path
    let kubeconfig = get_kubeconfig_path()?;
    let envs: Vec<(String, String)> = vec![];
    
    // Define the charts to install
    let charts: Vec<Box<dyn HelmChart>> = vec![
        // Infrastructure charts
        Box::new(PrometheusOperatorChart::default()),
        Box::new(ExternalDnsChart::local()),
        Box::new(NginxIngressChart::k3d_local()),
        
        // Application charts
        Box::new(PostgresqlChart::development()),
        Box::new(KafkaChart::strimzi_operator()),
    ];
    
    // Install each chart
    for chart in charts {
        let chart_name = chart.get_chart_info().name.clone();
        info!("Installing chart: {}", chart_name);
        
        match chart.run(&kubeconfig, &envs).await {
            Ok(Some(result)) => {
                if result.success {
                    info!("Successfully installed chart: {}", chart_name);
                    
                    // Log validation results
                    for validation in &result.validation_results {
                        debug!("  Validation '{}': {} - {}", 
                            validation.check_name,
                            if validation.passed { "PASSED" } else { "FAILED" },
                            validation.message
                        );
                    }
                } else {
                    return Err(FirestreamError::GeneralError(
                        format!("Failed to install chart {}: {:?}", chart_name, result.error)
                    ));
                }
            }
            Ok(None) => {
                debug!("Chart {} returned no result", chart_name);
            }
            Err(e) => {
                return Err(FirestreamError::GeneralError(
                    format!("Error installing chart {}: {}", chart_name, e)
                ));
            }
        }
    }
    
    Ok(())
}

/// Deploy a single chart using the lifecycle system
pub async fn deploy_chart_lifecycle(
    chart: Box<dyn HelmChart>,
    kubeconfig: Option<&Path>,
) -> Result<()> {
    let kubeconfig_path = if let Some(path) = kubeconfig {
        path.to_path_buf()
    } else {
        get_kubeconfig_path()?
    };
    
    let envs: Vec<(String, String)> = vec![];
    let chart_name = chart.get_chart_info().name.clone();
    
    info!("Deploying chart '{}' using lifecycle management", chart_name);
    
    match chart.run(&kubeconfig_path, &envs).await {
        Ok(Some(result)) => {
            if result.success {
                info!("Successfully deployed chart: {}", chart_name);
                Ok(())
            } else {
                Err(FirestreamError::GeneralError(
                    format!("Failed to deploy chart {}: {:?}", chart_name, result.error)
                ))
            }
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Deploy a Firestream app from a directory containing firestream.toml
pub async fn deploy_app(
    app_dir: impl AsRef<Path>,
    action: HelmAction,
    environment: Option<&str>,
) -> Result<()> {
    info!("Deploying Firestream app from {:?}", app_dir.as_ref());
    
    // Create app lifecycle from directory
    let app = AppLifecycle::from_directory(
        app_dir,
        action,
        environment.map(|e| e.to_string()),
    ).await?;
    
    // Deploy using standard lifecycle
    deploy_chart_lifecycle(Box::new(app), None).await?;
    
    Ok(())
}

/// Deploy all apps in a directory
pub async fn deploy_apps_directory(
    packages_dir: impl AsRef<Path>,
    action: HelmAction,
    environment: Option<&str>,
) -> Result<()> {
    let packages_dir = packages_dir.as_ref();
    info!("Deploying all apps from {:?}", packages_dir);
    
    // Read directory entries
    let mut entries = tokio::fs::read_dir(packages_dir).await?;
    let mut app_dirs = Vec::new();
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        // Check if it's a directory with firestream.toml
        if path.is_dir() {
            let manifest_path = path.join("firestream.toml");
            if manifest_path.exists() {
                app_dirs.push(path);
            }
        }
    }
    
    info!("Found {} apps to deploy", app_dirs.len());
    
    // Deploy each app
    for app_dir in app_dirs {
        match deploy_app(&app_dir, action, environment).await {
            Ok(_) => info!("Successfully deployed app from {:?}", app_dir),
            Err(e) => {
                eprintln!("Failed to deploy app from {:?}: {}", app_dir, e);
                // Continue with other apps
            }
        }
    }
    
    Ok(())
}

/// Deploy app with dependency resolution
pub async fn deploy_app_with_dependencies(
    app_dir: impl AsRef<Path>,
    packages_dir: impl AsRef<Path>,
    environment: Option<&str>,
) -> Result<()> {
    use crate::deploy::helm_lifecycle::app::{ManifestParser, manifest::LocalAppRegistry};
    
    let app_dir = app_dir.as_ref();
    let packages_dir = packages_dir.as_ref();
    
    info!("Deploying app with dependencies from {:?}", app_dir);
    
    // Parse manifest
    let manifest_path = app_dir.join("firestream.toml");
    let manifest = ManifestParser::load_with_environment(
        &manifest_path,
        environment,
    ).await?;
    
    // Create app registry
    let registry = LocalAppRegistry {
        base_dir: packages_dir.to_path_buf(),
    };
    
    // Resolve dependencies
    let dependencies = ManifestParser::resolve_dependencies(&manifest, &registry).await?;
    
    // Deploy dependencies first
    for dep in dependencies {
        info!("Deploying dependency: {} v{}", dep.name, dep.version);
        
        let dep_dir = packages_dir.join(&dep.name);
        deploy_app(&dep_dir, HelmAction::Deploy, environment).await?;
        
        if dep.should_wait {
            // Wait for dependency to be ready
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }
    
    // Deploy the main app
    deploy_app(app_dir, HelmAction::Deploy, environment).await
}

/// Get the kubeconfig path
fn get_kubeconfig_path() -> Result<PathBuf> {
    // First check KUBECONFIG env var
    if let Ok(kubeconfig) = std::env::var("KUBECONFIG") {
        return Ok(PathBuf::from(kubeconfig));
    }
    
    // Fall back to default location
    let home = dirs::home_dir()
        .ok_or_else(|| FirestreamError::GeneralError("Cannot find home directory".into()))?;
    
    Ok(home.join(".kube").join("config"))
}

// ========== Legacy functions below ==========

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
        HelmRepo::prometheus_community(),
        HelmRepo::ingress_nginx(),
        HelmRepo::strimzi(),
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

/// Install a Helm chart (legacy function - use lifecycle system instead)
pub async fn install_chart(release_name: &str, chart: &str, values_file: Option<&Path>) -> Result<()> {
    info!("Installing Helm chart '{}' as '{}'", chart, release_name);
    
    // Convert to lifecycle system
    let chart_info = ChartInfo {
        name: release_name.to_string(),
        chart: chart.to_string(),
        values_files: values_file.map(|p| vec![p.to_path_buf()]).unwrap_or_default(),
        ..Default::default()
    };
    
    let chart = CommonChart::new(chart_info);
    deploy_chart_lifecycle(Box::new(chart), None).await
}

/// Upgrade a Helm release (legacy function - use lifecycle system instead)
pub async fn upgrade_release(release_name: &str, chart: &str, values_file: Option<&Path>) -> Result<()> {
    info!("Upgrading Helm release '{}'", release_name);
    
    // Convert to lifecycle system
    let chart_info = ChartInfo {
        name: release_name.to_string(),
        chart: chart.to_string(),
        values_files: values_file.map(|p| vec![p.to_path_buf()]).unwrap_or_default(),
        ..Default::default()
    };
    
    let chart = CommonChart::new(chart_info);
    deploy_chart_lifecycle(Box::new(chart), None).await
}

/// Uninstall a Helm release (legacy function - use lifecycle system instead)
pub async fn uninstall_release(release_name: &str) -> Result<()> {
    info!("Uninstalling Helm release '{}'", release_name);
    
    // Convert to lifecycle system
    let chart_info = ChartInfo {
        name: release_name.to_string(),
        action: HelmAction::Destroy,
        ..Default::default()
    };
    
    let chart = CommonChart::new(chart_info);
    deploy_chart_lifecycle(Box::new(chart), None).await
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

// ========== Helper functions for state integration ==========

/// Create a chart from state deployment configuration
pub fn chart_from_deployment(
    deployment_name: &str,
    chart_ref: &str,
    values: serde_json::Value,
    namespace: &str,
) -> Result<Box<dyn HelmChart>> {
    // Parse chart reference (e.g., "bitnami/postgresql" or "postgresql")
    let parts: Vec<&str> = chart_ref.split('/').collect();
    let (repo, chart) = if parts.len() == 2 {
        (Some(parts[0].to_string()), parts[1].to_string())
    } else {
        (None, chart_ref.to_string())
    };
    
    // Convert JSON values to ChartSetValue vec
    let mut chart_values = Vec::new();
    if let Some(obj) = values.as_object() {
        flatten_json_to_helm_values("", obj, &mut chart_values);
    }
    
    let chart_info = ChartInfo {
        name: deployment_name.to_string(),
        repository: repo,
        chart,
        custom_namespace: Some(namespace.to_string()),
        values: chart_values,
        ..Default::default()
    };
    
    Ok(Box::new(CommonChart::new(chart_info)))
}

/// Flatten JSON object to Helm --set values
fn flatten_json_to_helm_values(
    prefix: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    output: &mut Vec<crate::deploy::helm_lifecycle::ChartSetValue>,
) {
    for (key, value) in obj {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };
        
        match value {
            serde_json::Value::Object(nested) => {
                flatten_json_to_helm_values(&full_key, nested, output);
            }
            serde_json::Value::Array(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    let array_key = format!("{}[{}]", full_key, i);
                    if let Some(obj) = item.as_object() {
                        flatten_json_to_helm_values(&array_key, obj, output);
                    } else {
                        output.push(crate::deploy::helm_lifecycle::ChartSetValue {
                            key: array_key,
                            value: item.to_string(),
                        });
                    }
                }
            }
            _ => {
                output.push(crate::deploy::helm_lifecycle::ChartSetValue {
                    key: full_key,
                    value: value.to_string(),
                });
            }
        }
    }
}

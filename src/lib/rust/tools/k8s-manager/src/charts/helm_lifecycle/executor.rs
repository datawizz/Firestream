//! Helm and Kubernetes command execution utilities

use crate::core::{FirestreamError, Result};
use super::types::*;
use std::path::Path;
use tokio::process::Command;
use tracing::{info, debug, warn};
use std::collections::HashMap;

/// Deploy a Helm chart
pub async fn helm_deploy(
    kubernetes_config: &Path,
    envs: &[(String, String)],
    chart_info: &ChartInfo,
) -> Result<()> {
    info!("Deploying Helm chart: {}", chart_info.name);
    
    let mut cmd = Command::new("helm");
    
    // Set kubeconfig
    cmd.env("KUBECONFIG", kubernetes_config);
    
    // Add environment variables
    for (key, value) in envs {
        cmd.env(key, value);
    }
    
    // Base command
    cmd.arg("upgrade")
        .arg("--install")
        .arg(&chart_info.name);
    
    // Chart source
    if let Some(path) = &chart_info.path {
        cmd.arg(path);
    } else if let Some(repo) = &chart_info.repository {
        cmd.arg(format!("{}/{}", repo, chart_info.chart));
    } else {
        cmd.arg(&chart_info.chart);
    }
    
    // Namespace
    let namespace_string = chart_info.namespace.as_str().to_string();
    let namespace = chart_info.custom_namespace.as_ref()
        .unwrap_or(&namespace_string);
    cmd.arg("--namespace").arg(namespace);
    
    // Create namespace if needed
    if chart_info.create_namespace {
        cmd.arg("--create-namespace");
    }
    
    // Version
    if let Some(version) = &chart_info.version {
        cmd.arg("--version").arg(version);
    }
    
    // Atomic deployment
    if chart_info.atomic {
        cmd.arg("--atomic");
    }
    
    // Force upgrade
    if chart_info.force_upgrade {
        cmd.arg("--force");
    }
    
    // Timeout
    cmd.arg("--timeout").arg(&chart_info.timeout);
    
    // Wait flags
    if chart_info.wait {
        cmd.arg("--wait");
    }
    
    if chart_info.wait_for_jobs {
        cmd.arg("--wait-for-jobs");
    }
    
    // Dry run
    if chart_info.dry_run {
        cmd.arg("--dry-run");
    }
    
    // Skip CRDs
    if chart_info.skip_crds {
        cmd.arg("--skip-crds");
    }
    
    // Disable hooks
    if chart_info.hooks_disabled {
        cmd.arg("--no-hooks");
    }
    
    // Add values files
    for values_file in &chart_info.values_files {
        cmd.arg("-f").arg(values_file);
    }
    
    // Create temporary values files for generated content
    let temp_dir = tempfile::tempdir()?;
    for generated in &chart_info.yaml_files_content {
        let temp_file = temp_dir.path().join(&generated.filename);
        std::fs::write(&temp_file, &generated.content)?;
        cmd.arg("-f").arg(&temp_file);
    }
    
    // Add set values
    for set_value in &chart_info.values {
        cmd.arg("--set").arg(format!("{}={}", set_value.key, set_value.value));
    }
    
    // Execute
    debug!("Executing: {:?}", cmd);
    let output = cmd.output().await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(FirestreamError::GeneralError(
            format!("Helm deploy failed for {}: {}\n{}", chart_info.name, stderr, stdout)
        ));
    }
    
    info!("Successfully deployed chart: {}", chart_info.name);
    Ok(())
}

/// Uninstall a Helm release
pub async fn helm_uninstall(
    kubernetes_config: &Path,
    envs: &[(String, String)],
    chart_info: &ChartInfo,
) -> Result<()> {
    info!("Uninstalling Helm release: {}", chart_info.name);
    
    let mut cmd = Command::new("helm");
    
    // Set kubeconfig
    cmd.env("KUBECONFIG", kubernetes_config);
    
    // Add environment variables
    for (key, value) in envs {
        cmd.env(key, value);
    }
    
    // Base command
    cmd.arg("uninstall").arg(&chart_info.name);
    
    // Namespace
    let namespace_string = chart_info.namespace.as_str().to_string();
    let namespace = chart_info.custom_namespace.as_ref()
        .unwrap_or(&namespace_string);
    cmd.arg("--namespace").arg(namespace);
    
    // Execute
    debug!("Executing: {:?}", cmd);
    let output = cmd.output().await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FirestreamError::GeneralError(
            format!("Helm uninstall failed for {}: {}", chart_info.name, stderr)
        ));
    }
    
    info!("Successfully uninstalled release: {}", chart_info.name);
    Ok(())
}

/// Check if a Helm release is deployed
pub async fn is_helm_release_deployed(
    kubernetes_config: &Path,
    envs: &[(String, String)],
    release_name: &str,
    namespace: &str,
) -> Result<bool> {
    let mut cmd = Command::new("helm");
    
    cmd.env("KUBECONFIG", kubernetes_config);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    
    cmd.arg("list")
        .arg("-n").arg(namespace)
        .arg("-o").arg("json");
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        warn!("Failed to list Helm releases");
        return Ok(false);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let releases: Vec<serde_json::Value> = serde_json::from_str(&stdout)
        .unwrap_or_default();
    
    Ok(releases.iter().any(|r| {
        r.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n == release_name)
            .unwrap_or(false)
    }))
}

/// Get Helm release status
pub async fn get_helm_release_status(
    kubernetes_config: &Path,
    envs: &[(String, String)],
    release_name: &str,
    namespace: &str,
) -> Result<HelmReleaseStatus> {
    let mut cmd = Command::new("helm");
    
    cmd.env("KUBECONFIG", kubernetes_config);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    
    cmd.arg("list")
        .arg("-n").arg(namespace)
        .arg("-o").arg("json")
        .arg("-f").arg(format!("^{}$", release_name));
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        return Err(FirestreamError::GeneralError(
            format!("Failed to get Helm release status for {}", release_name)
        ));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let releases: Vec<serde_json::Value> = serde_json::from_str(&stdout)?;
    
    let release = releases.first()
        .ok_or_else(|| FirestreamError::GeneralError(
            format!("Release {} not found", release_name)
        ))?;
    
    Ok(HelmReleaseStatus {
        name: release["name"].as_str().unwrap_or_default().to_string(),
        namespace: release["namespace"].as_str().unwrap_or_default().to_string(),
        revision: release["revision"].as_str().unwrap_or("0").parse().unwrap_or(0),
        status: release["status"].as_str().unwrap_or_default().to_string(),
        chart: release["chart"].as_str().unwrap_or_default().to_string(),
        app_version: release["app_version"].as_str().unwrap_or_default().to_string(),
    })
}

/// Get Kubernetes events
pub async fn get_kubernetes_events(
    kubernetes_config: &Path,
    envs: &[(String, String)],
    namespace: &str,
) -> Result<Vec<KubernetesEvent>> {
    let mut cmd = Command::new("kubectl");
    
    cmd.env("KUBECONFIG", kubernetes_config);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    
    cmd.arg("get").arg("events")
        .arg("-n").arg(namespace)
        .arg("-o").arg("json");
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        warn!("Failed to get Kubernetes events");
        return Ok(Vec::new());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let events_json: serde_json::Value = serde_json::from_str(&stdout)?;
    
    let mut events = Vec::new();
    if let Some(items) = events_json["items"].as_array() {
        for item in items {
            let event = KubernetesEvent {
                timestamp: item["firstTimestamp"].as_str().unwrap_or_default().to_string(),
                namespace: item["metadata"]["namespace"].as_str().unwrap_or_default().to_string(),
                name: item["metadata"]["name"].as_str().unwrap_or_default().to_string(),
                kind: item["involvedObject"]["kind"].as_str().unwrap_or_default().to_string(),
                reason: item["reason"].as_str().unwrap_or_default().to_string(),
                message: item["message"].as_str().unwrap_or_default().to_string(),
            };
            events.push(event);
        }
    }
    
    Ok(events)
}

/// Check if pods are running
pub async fn check_pods_running(
    kubernetes_config: &Path,
    envs: &[(String, String)],
    namespace: &str,
    release_name: &str,
) -> Result<ValidationResult> {
    let mut cmd = Command::new("kubectl");
    
    cmd.env("KUBECONFIG", kubernetes_config);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    
    cmd.arg("get").arg("pods")
        .arg("-n").arg(namespace)
        .arg("-l").arg(format!("app.kubernetes.io/instance={}", release_name))
        .arg("-o").arg("json");
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        return Ok(ValidationResult {
            check_name: "pods_running".to_string(),
            passed: false,
            message: "Failed to get pod status".to_string(),
            details: None,
        });
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pods_json: serde_json::Value = serde_json::from_str(&stdout)?;
    
    let mut total_pods = 0;
    let mut running_pods = 0;
    let mut pod_statuses = HashMap::new();
    
    if let Some(items) = pods_json["items"].as_array() {
        total_pods = items.len();
        
        for item in items {
            let pod_name = item["metadata"]["name"].as_str().unwrap_or_default();
            let phase = item["status"]["phase"].as_str().unwrap_or_default();
            
            if phase == "Running" {
                running_pods += 1;
            }
            
            pod_statuses.insert(pod_name.to_string(), phase.to_string());
        }
    }
    
    let passed = total_pods > 0 && running_pods == total_pods;
    let message = format!("{}/{} pods running", running_pods, total_pods);
    
    Ok(ValidationResult {
        check_name: "pods_running".to_string(),
        passed,
        message,
        details: Some(serde_json::to_value(pod_statuses)?),
    })
}

/// Execute a kubectl command
pub async fn execute_kubectl_command(
    kubernetes_config: &Path,
    envs: &[(String, String)],
    command: &str,
) -> Result<()> {
    info!("Executing kubectl command: {}", command);
    
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(FirestreamError::GeneralError("Empty kubectl command".to_string()));
    }
    
    let mut cmd = Command::new("kubectl");
    
    cmd.env("KUBECONFIG", kubernetes_config);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    
    for part in &parts {
        cmd.arg(part);
    }
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FirestreamError::GeneralError(
            format!("kubectl command failed: {}", stderr)
        ));
    }
    
    Ok(())
}

/// Delete a CRD
pub async fn kubectl_delete_crd(
    kubernetes_config: &Path,
    envs: &[(String, String)],
    crd_name: &str,
) -> Result<()> {
    info!("Deleting CRD: {}", crd_name);
    
    let mut cmd = Command::new("kubectl");
    
    cmd.env("KUBECONFIG", kubernetes_config);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    
    cmd.arg("delete").arg("crd").arg(crd_name).arg("--ignore-not-found");
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to delete CRD {}: {}", crd_name, stderr);
    }
    
    Ok(())
}

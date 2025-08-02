//! Observability features for K3D clusters

use crate::{K8sManagerError, Result, LogsConfig, DiagnosticsConfig};
use crate::providers::k3d::manager::K3dClusterManager;
use tokio::process::Command;
use std::collections::HashMap;


impl K3dClusterManager {
    /// Get logs from resources
    pub async fn get_logs(&self, config: &LogsConfig) -> Result<String> {
        let mut cmd = Command::new("kubectl");
        cmd.arg("logs");

        if let Some(ns) = &config.namespace {
            cmd.args(&["-n", ns]);
        }

        cmd.arg(format!("{}/{}", config.resource_type.as_str(), config.resource_name));

        if let Some(container) = &config.container {
            cmd.args(&["-c", container]);
        }

        if config.all_containers {
            cmd.arg("--all-containers");
        }

        if config.previous {
            cmd.arg("--previous");
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            return Err(K8sManagerError::GeneralError(
                format!("Failed to get logs: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Stream logs from resources
    pub async fn stream_logs(&self, config: &LogsConfig) -> Result<()> {
        let mut cmd = Command::new("kubectl");
        cmd.arg("logs");
        cmd.arg("-f"); // Follow

        if let Some(ns) = &config.namespace {
            cmd.args(&["-n", ns]);
        }

        cmd.arg(format!("{}/{}", config.resource_type.as_str(), config.resource_name));

        if let Some(container) = &config.container {
            cmd.args(&["-c", container]);
        }

        if config.all_containers {
            cmd.arg("--all-containers");
        }

        let mut child = cmd.spawn()?;
        let _ = child.wait().await?;

        Ok(())
    }

    /// Get cluster diagnostics
    pub async fn get_diagnostics(&self, config: &DiagnosticsConfig) -> Result<HashMap<String, String>> {
        let mut diagnostics = HashMap::new();

        if config.include_all || config.include_nodes {
            let output = Command::new("kubectl")
                .args(&["get", "nodes", "-o", "wide"])
                .output()
                .await?;

            if output.status.success() {
                diagnostics.insert("nodes".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
            }
        }

        if config.include_all || config.include_pods {
            let mut cmd = Command::new("kubectl");
            cmd.args(&["get", "pods", "-o", "wide"]);

            if let Some(ns) = &config.namespace {
                cmd.args(&["-n", ns]);
            } else {
                cmd.arg("--all-namespaces");
            }

            let output = cmd.output().await?;

            if output.status.success() {
                diagnostics.insert("pods".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
            }
        }

        if config.include_all || config.include_services {
            let mut cmd = Command::new("kubectl");
            cmd.args(&["get", "svc", "-o", "wide"]);

            if let Some(ns) = &config.namespace {
                cmd.args(&["-n", ns]);
            } else {
                cmd.arg("--all-namespaces");
            }

            let output = cmd.output().await?;

            if output.status.success() {
                diagnostics.insert("services".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
            }
        }

        if config.include_all || config.include_events {
            let mut cmd = Command::new("kubectl");
            cmd.args(&["get", "events", "--sort-by=.metadata.creationTimestamp"]);

            if let Some(ns) = &config.namespace {
                cmd.args(&["-n", ns]);
            } else {
                cmd.arg("--all-namespaces");
            }

            let output = cmd.output().await?;

            if output.status.success() {
                diagnostics.insert("events".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
            }
        }

        Ok(diagnostics)
    }

    /// Get cluster metrics
    pub async fn get_metrics(&self) -> Result<HashMap<String, serde_json::Value>> {
        // K3D doesn't have metrics server by default
        // This would need to be implemented after installing metrics-server
        Ok(HashMap::new())
    }
}

//! Type definitions for Helm lifecycle management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Helm action to perform
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HelmAction {
    Deploy,
    Destroy,
    Skip,
}

/// Kubernetes namespaces for Helm charts
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HelmChartNamespace {
    KubeSystem,
    Default,
    Prometheus,
    Logging,
    CertManager,
    NginxIngress,
    Firestream,
    Custom(u32), // For custom namespaces, store an index
}

impl HelmChartNamespace {
    pub fn as_str(&self) -> &str {
        match self {
            HelmChartNamespace::KubeSystem => "kube-system",
            HelmChartNamespace::Default => "default",
            HelmChartNamespace::Prometheus => "prometheus",
            HelmChartNamespace::Logging => "logging",
            HelmChartNamespace::CertManager => "cert-manager",
            HelmChartNamespace::NginxIngress => "nginx-ingress",
            HelmChartNamespace::Firestream => "firestream",
            HelmChartNamespace::Custom(_) => "default", // Handle custom namespaces specially
        }
    }
}

/// Chart set value for Helm --set flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSetValue {
    pub key: String,
    pub value: String,
}

/// Generated chart values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartValuesGenerated {
    pub filename: String,
    pub content: String,
}

/// Version information for breaking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingVersion {
    pub version: String,
    pub requires_uninstall: bool,
    pub pre_upgrade_commands: Vec<String>,
}

/// Chart information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartInfo {
    pub name: String,
    pub repository: Option<String>,
    pub chart: String,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub namespace: HelmChartNamespace,
    pub custom_namespace: Option<String>,
    pub action: HelmAction,
    pub atomic: bool,
    pub force_upgrade: bool,
    pub create_namespace: bool,
    pub last_breaking_version_requiring_restart: Option<BreakingVersion>,
    pub timeout: String,
    pub dry_run: bool,
    pub wait: bool,
    pub wait_for_jobs: bool,
    pub values: Vec<ChartSetValue>,
    pub values_files: Vec<PathBuf>,
    pub yaml_files_content: Vec<ChartValuesGenerated>,
    pub depends_on: Vec<String>,
    pub hooks_disabled: bool,
    pub skip_crds: bool,
}

impl Default for ChartInfo {
    fn default() -> Self {
        ChartInfo {
            name: "undefined".to_string(),
            repository: None,
            chart: "undefined".to_string(),
            version: None,
            path: None,
            namespace: HelmChartNamespace::Default,
            custom_namespace: None,
            action: HelmAction::Deploy,
            atomic: true,
            force_upgrade: false,
            create_namespace: true,
            last_breaking_version_requiring_restart: None,
            timeout: "300s".to_string(),
            dry_run: false,
            wait: true,
            wait_for_jobs: true,
            values: Vec::new(),
            values_files: Vec::new(),
            yaml_files_content: Vec::new(),
            depends_on: Vec::new(),
            hooks_disabled: false,
            skip_crds: false,
        }
    }
}

/// Payload that can be passed between lifecycle methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPayload {
    pub data: HashMap<String, serde_json::Value>,
}

impl ChartPayload {
    pub fn new() -> Self {
        ChartPayload {
            data: HashMap::new(),
        }
    }

    pub fn insert<T: Serialize>(&mut self, key: &str, value: T) -> crate::core::Result<()> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| crate::core::FirestreamError::GeneralError(
                format!("Failed to serialize value: {}", e)
            ))?;
        self.data.insert(key.to_string(), json_value);
        Ok(())
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Helm release status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmReleaseStatus {
    pub name: String,
    pub namespace: String,
    pub revision: u32,
    pub status: String,
    pub chart: String,
    pub app_version: String,
}

/// Kubernetes event for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesEvent {
    pub timestamp: String,
    pub namespace: String,
    pub name: String,
    pub kind: String,
    pub reason: String,
    pub message: String,
}

/// Validation check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub check_name: String,
    pub passed: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Chart deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDeploymentResult {
    pub chart_name: String,
    pub success: bool,
    pub release_info: Option<HelmReleaseStatus>,
    pub validation_results: Vec<ValidationResult>,
    pub events: Vec<KubernetesEvent>,
    pub error: Option<String>,
}

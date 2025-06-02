//! Prometheus Operator chart implementation

use crate::core::Result;
use crate::deploy::helm_lifecycle::{
    HelmChart, ChartInfo, ChartPayload, HelmAction, ValidationResult,
    BaseHelmChart, executor::*,
};
use std::path::Path;
use tracing::{info, warn};
use async_trait::async_trait;

/// Prometheus Operator chart with CRD management
pub struct PrometheusOperatorChart {
    pub chart_info: ChartInfo,
}

impl PrometheusOperatorChart {
    pub fn new(chart_info: ChartInfo) -> Self {
        Self { chart_info }
    }
    
    /// Default configuration for Prometheus Operator
    pub fn default() -> Self {
        use crate::deploy::helm_lifecycle::{HelmChartNamespace, ChartSetValue, values_path};
        
        let chart_info = ChartInfo {
            name: "prometheus-operator".to_string(),
            repository: Some("prometheus-community".to_string()),
            chart: "kube-prometheus-stack".to_string(),
            version: Some("45.7.1".to_string()),
            namespace: HelmChartNamespace::Prometheus,
            timeout: "600s".to_string(),
            values_files: vec![values_path("prometheus-operator.yaml")],
            values: vec![
                ChartSetValue {
                    key: "prometheus.prometheusSpec.retention".to_string(),
                    value: "30d".to_string(),
                },
                ChartSetValue {
                    key: "prometheus.prometheusSpec.resources.requests.memory".to_string(),
                    value: "1Gi".to_string(),
                },
                ChartSetValue {
                    key: "prometheus.prometheusSpec.resources.limits.memory".to_string(),
                    value: "2Gi".to_string(),
                },
                ChartSetValue {
                    key: "grafana.adminPassword".to_string(),
                    value: "admin".to_string(),
                },
            ],
            ..Default::default()
        };
        
        Self { chart_info }
    }
}

#[async_trait]
impl HelmChart for PrometheusOperatorChart {
    fn get_chart_info(&self) -> &ChartInfo {
        &self.chart_info
    }
    
    fn get_chart_info_mut(&mut self) -> &mut ChartInfo {
        &mut self.chart_info
    }
    
    async fn exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        let chart_info = self.get_chart_info();
        
        match chart_info.action {
            HelmAction::Deploy => {
                // Check and handle breaking changes
                if let Some(breaking_version) = &chart_info.last_breaking_version_requiring_restart {
                    if breaking_version.requires_uninstall {
                        warn!("Prometheus Operator requires uninstall for breaking changes");
                        
                        // Check if currently deployed
                        if self.is_deployed(kubernetes_config, envs).await? {
                            info!("Uninstalling existing Prometheus Operator for upgrade");
                            helm_uninstall(kubernetes_config, envs, chart_info).await?;
                            
                            // Clean up CRDs if needed
                            self.cleanup_crds(kubernetes_config, envs).await?;
                            
                            // Wait a bit for cleanup
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        }
                    }
                }
                
                // Deploy the chart
                helm_deploy(kubernetes_config, envs, chart_info).await?;
            }
            
            HelmAction::Destroy => {
                if self.is_deployed(kubernetes_config, envs).await? {
                    // Uninstall the release
                    helm_uninstall(kubernetes_config, envs, chart_info).await?;
                    
                    // Clean up CRDs
                    self.cleanup_crds(kubernetes_config, envs).await?;
                } else {
                    info!("Prometheus Operator is not deployed, skipping uninstall");
                }
            }
            
            HelmAction::Skip => {
                info!("Skipping Prometheus Operator deployment");
            }
        }
        
        Ok(payload)
    }
    
    async fn pre_exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        mut payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        info!("Pre-exec: Preparing for Prometheus Operator deployment");
        
        // Add prometheus-community repo if not already added
        let mut cmd = tokio::process::Command::new("helm");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("repo").arg("add")
            .arg("prometheus-community")
            .arg("https://prometheus-community.github.io/helm-charts")
            .arg("--force-update");
        
        let _ = cmd.output().await?;
        
        // Update repo
        let mut cmd = tokio::process::Command::new("helm");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("repo").arg("update").arg("prometheus-community");
        let _ = cmd.output().await?;
        
        // Store pre-exec info in payload
        if let Some(ref mut p) = payload {
            p.insert("repo_added", true)?;
        }
        
        Ok(payload)
    }
    
    async fn post_exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        mut payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        if self.get_chart_info().action == HelmAction::Deploy {
            info!("Post-exec: Waiting for Prometheus Operator to be ready");
            
            // Wait for the operator to be ready
            let mut cmd = tokio::process::Command::new("kubectl");
            cmd.env("KUBECONFIG", kubernetes_config);
            for (key, value) in envs {
                cmd.env(key, value);
            }
            
            cmd.arg("wait")
                .arg("--for=condition=ready")
                .arg("pod")
                .arg("-l").arg("app.kubernetes.io/name=prometheus-operator")
                .arg("-n").arg(self.get_namespace())
                .arg("--timeout=300s");
            
            let output = cmd.output().await?;
            
            if !output.status.success() {
                warn!("Prometheus Operator pods not ready after timeout");
            }
            
            // Store post-exec status
            if let Some(ref mut p) = payload {
                p.insert("operator_ready", output.status.success())?;
            }
        }
        
        Ok(payload)
    }
    
    async fn validate(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();
        
        // Use base validation first
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        let mut base_results = base.validate(kubernetes_config, envs, payload).await?;
        results.append(&mut base_results);
        
        if self.get_chart_info().action == HelmAction::Deploy {
            // Check CRDs are installed
            let crd_check = self.check_crds_installed(kubernetes_config, envs).await?;
            results.push(crd_check);
            
            // Check Prometheus instances
            let prometheus_check = self.check_prometheus_instances(kubernetes_config, envs).await?;
            results.push(prometheus_check);
            
            // Check Grafana is accessible
            let grafana_check = self.check_grafana_ready(kubernetes_config, envs).await?;
            results.push(grafana_check);
        }
        
        Ok(results)
    }
    
    async fn collect_events(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<Vec<crate::deploy::helm_lifecycle::KubernetesEvent>> {
        get_kubernetes_events(kubernetes_config, envs, &self.get_namespace()).await
    }
    
    async fn get_release_info(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<crate::deploy::helm_lifecycle::HelmReleaseStatus> {
        get_helm_release_status(kubernetes_config, envs, &self.chart_info.name, &self.get_namespace()).await
    }
    
    async fn is_deployed(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<bool> {
        is_helm_release_deployed(kubernetes_config, envs, &self.chart_info.name, &self.get_namespace()).await
    }
}

impl PrometheusOperatorChart {
    /// Clean up Prometheus CRDs
    async fn cleanup_crds(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<()> {
        info!("Cleaning up Prometheus CRDs");
        
        let prometheus_crds = [
            "prometheuses.monitoring.coreos.com",
            "prometheusrules.monitoring.coreos.com",
            "servicemonitors.monitoring.coreos.com",
            "podmonitors.monitoring.coreos.com",
            "alertmanagers.monitoring.coreos.com",
            "thanosrulers.monitoring.coreos.com",
            "probes.monitoring.coreos.com",
        ];
        
        for crd in &prometheus_crds {
            if let Err(e) = kubectl_delete_crd(kubernetes_config, envs, crd).await {
                warn!("Failed to delete CRD {}: {}", crd, e);
            }
        }
        
        Ok(())
    }
    
    /// Check if Prometheus CRDs are installed
    async fn check_crds_installed(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<ValidationResult> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get").arg("crd").arg("prometheuses.monitoring.coreos.com");
        
        let output = cmd.output().await?;
        
        Ok(ValidationResult {
            check_name: "prometheus_crds".to_string(),
            passed: output.status.success(),
            message: if output.status.success() {
                "Prometheus CRDs are installed".to_string()
            } else {
                "Prometheus CRDs are not installed".to_string()
            },
            details: None,
        })
    }
    
    /// Check Prometheus instances
    async fn check_prometheus_instances(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<ValidationResult> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get").arg("prometheus")
            .arg("-n").arg(self.get_namespace())
            .arg("-o").arg("json");
        
        let output = cmd.output().await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: serde_json::Value = serde_json::from_str(&stdout)?;
            
            let instance_count = json["items"].as_array().map(|a| a.len()).unwrap_or(0);
            
            Ok(ValidationResult {
                check_name: "prometheus_instances".to_string(),
                passed: instance_count > 0,
                message: format!("{} Prometheus instances found", instance_count),
                details: Some(json),
            })
        } else {
            Ok(ValidationResult {
                check_name: "prometheus_instances".to_string(),
                passed: false,
                message: "Failed to get Prometheus instances".to_string(),
                details: None,
            })
        }
    }
    
    /// Check if Grafana is ready
    async fn check_grafana_ready(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<ValidationResult> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get").arg("pod")
            .arg("-n").arg(self.get_namespace())
            .arg("-l").arg("app.kubernetes.io/name=grafana")
            .arg("-o").arg("json");
        
        let output = cmd.output().await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: serde_json::Value = serde_json::from_str(&stdout)?;
            
            let empty_vec = Vec::new();
            let pods = json["items"].as_array().unwrap_or(&empty_vec);
            let ready_pods = pods.iter().filter(|pod| {
                pod["status"]["phase"].as_str() == Some("Running")
            }).count();
            
            Ok(ValidationResult {
                check_name: "grafana_ready".to_string(),
                passed: ready_pods > 0,
                message: format!("{}/{} Grafana pods ready", ready_pods, pods.len()),
                details: None,
            })
        } else {
            Ok(ValidationResult {
                check_name: "grafana_ready".to_string(),
                passed: false,
                message: "Failed to check Grafana status".to_string(),
                details: None,
            })
        }
    }
}

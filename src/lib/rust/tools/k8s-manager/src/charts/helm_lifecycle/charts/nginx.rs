//! NGINX Ingress Controller chart implementation

use crate::core::Result;
use crate::deploy::helm_lifecycle::{
    HelmChart, ChartInfo, ChartSetValue, HelmChartNamespace,
    ChartPayload, ValidationResult, BaseHelmChart,
    values_path,
};
use std::path::Path;
use tracing::info;
use async_trait::async_trait;

/// NGINX Ingress Controller chart
pub struct NginxIngressChart {
    pub chart_info: ChartInfo,
}

impl NginxIngressChart {
    /// Create NGINX Ingress Controller with default configuration
    pub fn default() -> Self {
        let chart_info = ChartInfo {
            name: "nginx-ingress".to_string(),
            repository: Some("ingress-nginx".to_string()),
            chart: "ingress-nginx".to_string(),
            version: Some("4.7.1".to_string()),
            namespace: HelmChartNamespace::NginxIngress,
            values_files: vec![values_path("nginx-ingress.yaml")],
            values: vec![
                // Controller configuration
                ChartSetValue {
                    key: "controller.service.type".to_string(),
                    value: "LoadBalancer".to_string(),
                },
                ChartSetValue {
                    key: "controller.metrics.enabled".to_string(),
                    value: "true".to_string(),
                },
                ChartSetValue {
                    key: "controller.metrics.serviceMonitor.enabled".to_string(),
                    value: "true".to_string(),
                },
                // Resource limits
                ChartSetValue {
                    key: "controller.resources.limits.cpu".to_string(),
                    value: "200m".to_string(),
                },
                ChartSetValue {
                    key: "controller.resources.requests.cpu".to_string(),
                    value: "100m".to_string(),
                },
                ChartSetValue {
                    key: "controller.resources.limits.memory".to_string(),
                    value: "256Mi".to_string(),
                },
                ChartSetValue {
                    key: "controller.resources.requests.memory".to_string(),
                    value: "128Mi".to_string(),
                },
            ],
            ..Default::default()
        };
        
        Self { chart_info }
    }
    
    /// Create NGINX for local K3D cluster
    pub fn k3d_local() -> Self {
        let mut chart = Self::default();
        
        // Override service type for K3D
        chart.chart_info.values.retain(|v| !v.key.starts_with("controller.service"));
        chart.chart_info.values.push(ChartSetValue {
            key: "controller.service.type".to_string(),
            value: "ClusterIP".to_string(),
        });
        
        // Enable host network for K3D
        chart.chart_info.values.push(ChartSetValue {
            key: "controller.hostNetwork".to_string(),
            value: "true".to_string(),
        });
        
        // Set ingress class as default
        chart.chart_info.values.push(ChartSetValue {
            key: "controller.ingressClassResource.default".to_string(),
            value: "true".to_string(),
        });
        
        chart
    }
}

#[async_trait]
impl HelmChart for NginxIngressChart {
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
        // Use base implementation
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        base.exec(kubernetes_config, envs, payload).await
    }
    
    async fn pre_exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        info!("Pre-exec: Adding ingress-nginx Helm repository");
        
        // Add ingress-nginx repo
        let mut cmd = tokio::process::Command::new("helm");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("repo").arg("add")
            .arg("ingress-nginx")
            .arg("https://kubernetes.github.io/ingress-nginx")
            .arg("--force-update");
        
        let _ = cmd.output().await?;
        
        // Update repo
        let mut cmd = tokio::process::Command::new("helm");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("repo").arg("update").arg("ingress-nginx");
        let _ = cmd.output().await?;
        
        Ok(payload)
    }
    
    async fn validate(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();
        
        // Use base validation
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        let mut base_results = base.validate(kubernetes_config, envs, payload).await?;
        results.append(&mut base_results);
        
        // Additional validation: Check ingress class
        if self.get_chart_info().action == crate::deploy::helm_lifecycle::HelmAction::Deploy {
            let ingress_class_check = self.check_ingress_class(kubernetes_config, envs).await?;
            results.push(ingress_class_check);
            
            // Check if LoadBalancer has external IP (for non-local environments)
            if !self.chart_info.values.iter().any(|v| v.key == "controller.hostNetwork" && v.value == "true") {
                let lb_check = self.check_load_balancer(kubernetes_config, envs).await?;
                results.push(lb_check);
            }
        }
        
        Ok(results)
    }
    
    async fn collect_events(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<Vec<crate::deploy::helm_lifecycle::KubernetesEvent>> {
        crate::deploy::helm_lifecycle::executor::get_kubernetes_events(
            kubernetes_config,
            envs,
            &self.get_namespace()
        ).await
    }
    
    async fn get_release_info(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<crate::deploy::helm_lifecycle::HelmReleaseStatus> {
        crate::deploy::helm_lifecycle::executor::get_helm_release_status(
            kubernetes_config,
            envs,
            &self.chart_info.name,
            &self.get_namespace()
        ).await
    }
    
    async fn is_deployed(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<bool> {
        crate::deploy::helm_lifecycle::executor::is_helm_release_deployed(
            kubernetes_config,
            envs,
            &self.chart_info.name,
            &self.get_namespace()
        ).await
    }
}

impl NginxIngressChart {
    /// Check if ingress class is available
    async fn check_ingress_class(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<ValidationResult> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get").arg("ingressclass").arg("nginx");
        
        let output = cmd.output().await?;
        
        Ok(ValidationResult {
            check_name: "ingress_class".to_string(),
            passed: output.status.success(),
            message: if output.status.success() {
                "NGINX ingress class is available".to_string()
            } else {
                "NGINX ingress class not found".to_string()
            },
            details: None,
        })
    }
    
    /// Check LoadBalancer status
    async fn check_load_balancer(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<ValidationResult> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get").arg("svc")
            .arg("-n").arg(self.get_namespace())
            .arg(&format!("{}-controller", self.chart_info.name))
            .arg("-o").arg("json");
        
        let output = cmd.output().await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: serde_json::Value = serde_json::from_str(&stdout)?;
            
            let external_ip = json["status"]["loadBalancer"]["ingress"][0]["ip"].as_str()
                .or_else(|| json["status"]["loadBalancer"]["ingress"][0]["hostname"].as_str());
            
            Ok(ValidationResult {
                check_name: "load_balancer_ip".to_string(),
                passed: external_ip.is_some(),
                message: if let Some(ip) = external_ip {
                    format!("LoadBalancer has external IP: {}", ip)
                } else {
                    "LoadBalancer external IP pending".to_string()
                },
                details: Some(json["status"]["loadBalancer"].clone()),
            })
        } else {
            Ok(ValidationResult {
                check_name: "load_balancer_ip".to_string(),
                passed: false,
                message: "Failed to get LoadBalancer service".to_string(),
                details: None,
            })
        }
    }
}

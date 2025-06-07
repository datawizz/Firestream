//! Apache Kafka chart implementation

use crate::core::Result;
use crate::deploy::helm_lifecycle::{
    HelmChart, ChartInfo, ChartSetValue, HelmChartNamespace,
    ChartPayload, ValidationResult, BaseHelmChart,
    values_path, HelmAction,
};
use std::path::Path;
use tracing::{info, warn};
use async_trait::async_trait;

/// Kafka chart with Strimzi operator
pub struct KafkaChart {
    pub chart_info: ChartInfo,
}

impl KafkaChart {
    /// Create Kafka chart using Strimzi operator
    pub fn strimzi_operator() -> Self {
        let chart_info = ChartInfo {
            name: "strimzi-kafka-operator".to_string(),
            repository: Some("strimzi".to_string()),
            chart: "strimzi-kafka-operator".to_string(),
            version: Some("0.38.0".to_string()),
            namespace: HelmChartNamespace::Default,
            values_files: vec![values_path("strimzi-operator.yaml")],
            values: vec![
                ChartSetValue {
                    key: "watchAnyNamespace".to_string(),
                    value: "false".to_string(),
                },
                ChartSetValue {
                    key: "resources.limits.cpu".to_string(),
                    value: "500m".to_string(),
                },
                ChartSetValue {
                    key: "resources.requests.cpu".to_string(),
                    value: "200m".to_string(),
                },
                ChartSetValue {
                    key: "resources.limits.memory".to_string(),
                    value: "384Mi".to_string(),
                },
                ChartSetValue {
                    key: "resources.requests.memory".to_string(),
                    value: "256Mi".to_string(),
                },
            ],
            ..Default::default()
        };
        
        Self { chart_info }
    }
    
    /// Create Kafka using Bitnami chart (simpler but less flexible)
    pub fn bitnami() -> CommonChart {
        use crate::deploy::helm_lifecycle::CommonChart;
        
        let chart_info = ChartInfo {
            name: "kafka".to_string(),
            repository: Some("bitnami".to_string()),
            chart: "kafka".to_string(),
            version: Some("26.8.3".to_string()),
            namespace: HelmChartNamespace::Default,
            values_files: vec![values_path("kafka.yaml")],
            values: vec![
                // Broker configuration
                ChartSetValue {
                    key: "broker.replicaCount".to_string(),
                    value: "3".to_string(),
                },
                ChartSetValue {
                    key: "broker.resources.limits.cpu".to_string(),
                    value: "1000m".to_string(),
                },
                ChartSetValue {
                    key: "broker.resources.requests.cpu".to_string(),
                    value: "500m".to_string(),
                },
                ChartSetValue {
                    key: "broker.resources.limits.memory".to_string(),
                    value: "2Gi".to_string(),
                },
                ChartSetValue {
                    key: "broker.resources.requests.memory".to_string(),
                    value: "1Gi".to_string(),
                },
                // Storage
                ChartSetValue {
                    key: "broker.persistence.enabled".to_string(),
                    value: "true".to_string(),
                },
                ChartSetValue {
                    key: "broker.persistence.size".to_string(),
                    value: "20Gi".to_string(),
                },
                // ZooKeeper mode (will be deprecated in favor of KRaft)
                ChartSetValue {
                    key: "zookeeper.enabled".to_string(),
                    value: "true".to_string(),
                },
                ChartSetValue {
                    key: "zookeeper.replicaCount".to_string(),
                    value: "3".to_string(),
                },
                // Metrics
                ChartSetValue {
                    key: "metrics.kafka.enabled".to_string(),
                    value: "true".to_string(),
                },
                ChartSetValue {
                    key: "metrics.jmx.enabled".to_string(),
                    value: "true".to_string(),
                },
            ],
            depends_on: vec!["zookeeper".to_string()],
            ..Default::default()
        };
        
        CommonChart::new(chart_info)
    }
}

#[async_trait]
impl HelmChart for KafkaChart {
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
        info!("Pre-exec: Adding Strimzi Helm repository");
        
        // Add Strimzi repo
        let mut cmd = tokio::process::Command::new("helm");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("repo").arg("add")
            .arg("strimzi")
            .arg("https://strimzi.io/charts/")
            .arg("--force-update");
        
        let _ = cmd.output().await?;
        
        // Update repo
        let mut cmd = tokio::process::Command::new("helm");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("repo").arg("update").arg("strimzi");
        let _ = cmd.output().await?;
        
        Ok(payload)
    }
    
    async fn post_exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        mut payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        if self.get_chart_info().action == HelmAction::Deploy {
            info!("Post-exec: Creating Kafka cluster CR");
            
            // After operator is installed, we need to create a Kafka cluster
            // This would typically be done through a CR (Custom Resource)
            // For now, we'll just wait for the operator to be ready
            
            let mut cmd = tokio::process::Command::new("kubectl");
            cmd.env("KUBECONFIG", kubernetes_config);
            for (key, value) in envs {
                cmd.env(key, value);
            }
            
            cmd.arg("wait")
                .arg("--for=condition=ready")
                .arg("pod")
                .arg("-l").arg("name=strimzi-cluster-operator")
                .arg("-n").arg(self.get_namespace())
                .arg("--timeout=300s");
            
            let output = cmd.output().await?;
            
            if !output.status.success() {
                warn!("Strimzi operator not ready after timeout");
            } else {
                // Here you would apply a Kafka CR
                // kubectl apply -f kafka-cluster.yaml
                info!("Strimzi operator is ready, Kafka cluster CRs can now be created");
            }
            
            if let Some(ref mut p) = payload {
                p.insert("operator_ready", output.status.success())?;
                p.insert("next_step", "Create Kafka cluster using Strimzi CRs")?;
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
        
        // Use base validation
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        let mut base_results = base.validate(kubernetes_config, envs, payload).await?;
        results.append(&mut base_results);
        
        if self.get_chart_info().action == HelmAction::Deploy {
            // Check if Strimzi CRDs are installed
            let crd_check = self.check_strimzi_crds(kubernetes_config, envs).await?;
            results.push(crd_check);
            
            // Check operator deployment
            let operator_check = self.check_operator_deployment(kubernetes_config, envs).await?;
            results.push(operator_check);
        }
        
        Ok(results)
    }
    
    async fn on_deploy_failure(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        warn!("Kafka deployment failed, collecting diagnostics");
        
        // Get operator logs
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("logs")
            .arg("-l").arg("name=strimzi-cluster-operator")
            .arg("-n").arg(self.get_namespace())
            .arg("--tail=50");
        
        if let Ok(output) = cmd.output().await {
            let logs = String::from_utf8_lossy(&output.stdout);
            warn!("Operator logs:\n{}", logs);
        }
        
        // Call base failure handler
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        base.on_deploy_failure(kubernetes_config, envs, payload).await
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

impl KafkaChart {
    /// Check if Strimzi CRDs are installed
    async fn check_strimzi_crds(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<ValidationResult> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get").arg("crd").arg("kafkas.kafka.strimzi.io");
        
        let output = cmd.output().await?;
        
        Ok(ValidationResult {
            check_name: "strimzi_crds".to_string(),
            passed: output.status.success(),
            message: if output.status.success() {
                "Strimzi CRDs are installed".to_string()
            } else {
                "Strimzi CRDs are not installed".to_string()
            },
            details: None,
        })
    }
    
    /// Check operator deployment status
    async fn check_operator_deployment(&self, kubernetes_config: &Path, envs: &[(String, String)]) -> Result<ValidationResult> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get").arg("deployment")
            .arg("-l").arg("name=strimzi-cluster-operator")
            .arg("-n").arg(self.get_namespace())
            .arg("-o").arg("json");
        
        let output = cmd.output().await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: serde_json::Value = serde_json::from_str(&stdout)?;
            
            let ready = json["items"][0]["status"]["readyReplicas"].as_u64().unwrap_or(0);
            let desired = json["items"][0]["status"]["replicas"].as_u64().unwrap_or(1);
            
            Ok(ValidationResult {
                check_name: "operator_deployment".to_string(),
                passed: ready == desired && ready > 0,
                message: format!("Strimzi operator: {}/{} replicas ready", ready, desired),
                details: Some(json["items"][0]["status"].clone()),
            })
        } else {
            Ok(ValidationResult {
                check_name: "operator_deployment".to_string(),
                passed: false,
                message: "Failed to check operator deployment".to_string(),
                details: None,
            })
        }
    }
}

// Re-export CommonChart for Bitnami Kafka
pub use crate::deploy::helm_lifecycle::CommonChart;

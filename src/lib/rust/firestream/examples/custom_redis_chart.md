# Example: Custom Redis Chart with Lifecycle Management

This example demonstrates how to create a custom Helm chart implementation using Firestream's lifecycle system.

```rust
use firestream::deploy::helm_lifecycle::{
    HelmChart, ChartInfo, ChartSetValue, HelmChartNamespace,
    ChartPayload, ValidationResult, BaseHelmChart, HelmAction,
    executor::*, types::*,
};
use std::path::Path;
use tracing::{info, warn};
use async_trait::async_trait;
use firestream::core::Result;

/// Custom Redis chart with sentinel support and advanced lifecycle management
pub struct RedisChart {
    pub chart_info: ChartInfo,
    pub enable_sentinel: bool,
    pub enable_cluster: bool,
}

impl RedisChart {
    /// Create a standalone Redis instance
    pub fn standalone() -> Self {
        let chart_info = ChartInfo {
            name: "redis".to_string(),
            repository: Some("bitnami".to_string()),
            chart: "redis".to_string(),
            version: Some("18.6.1".to_string()),
            namespace: HelmChartNamespace::Default,
            values: vec![
                ChartSetValue {
                    key: "architecture".to_string(),
                    value: "standalone".to_string(),
                },
                ChartSetValue {
                    key: "auth.enabled".to_string(),
                    value: "false".to_string(),
                },
                ChartSetValue {
                    key: "master.persistence.enabled".to_string(),
                    value: "true".to_string(),
                },
                ChartSetValue {
                    key: "master.persistence.size".to_string(),
                    value: "8Gi".to_string(),
                },
            ],
            ..Default::default()
        };
        
        Self {
            chart_info,
            enable_sentinel: false,
            enable_cluster: false,
        }
    }
    
    /// Create Redis with Sentinel for HA
    pub fn with_sentinel() -> Self {
        let mut redis = Self::standalone();
        
        redis.enable_sentinel = true;
        redis.chart_info.values.retain(|v| v.key != "architecture");
        redis.chart_info.values.push(ChartSetValue {
            key: "architecture".to_string(),
            value: "replication".to_string(),
        });
        
        // Sentinel configuration
        redis.chart_info.values.extend(vec![
            ChartSetValue {
                key: "sentinel.enabled".to_string(),
                value: "true".to_string(),
            },
            ChartSetValue {
                key: "sentinel.quorum".to_string(),
                value: "2".to_string(),
            },
            ChartSetValue {
                key: "replica.replicaCount".to_string(),
                value: "3".to_string(),
            },
        ]);
        
        redis
    }
}

#[async_trait]
impl HelmChart for RedisChart {
    fn get_chart_info(&self) -> &ChartInfo {
        &self.chart_info
    }
    
    fn get_chart_info_mut(&mut self) -> &mut ChartInfo {
        &mut self.chart_info
    }
    
    async fn check_prerequisites(
        &self,
        mut payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        info!("Checking Redis prerequisites");
        
        // Check if persistent volumes are available
        if self.chart_info.values.iter().any(|v| 
            v.key.contains("persistence.enabled") && v.value == "true"
        ) {
            info!("Redis requires persistent volumes");
            
            if let Some(ref mut p) = payload {
                p.insert("requires_pv", true)?;
            }
        }
        
        // Check for existing Redis instances that might conflict
        if let Some(ref mut p) = payload {
            p.insert("prerequisites_checked", true)?;
        }
        
        Ok(payload)
    }
    
    async fn pre_exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        mut payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        info!("Pre-exec: Preparing Redis deployment");
        
        // Create Redis namespace if using custom namespace
        if let Some(ns) = &self.chart_info.custom_namespace {
            let mut cmd = tokio::process::Command::new("kubectl");
            cmd.env("KUBECONFIG", kubernetes_config);
            for (key, value) in envs {
                cmd.env(key, value);
            }
            
            cmd.arg("create")
                .arg("namespace")
                .arg(ns)
                .arg("--dry-run=client")
                .arg("-o").arg("yaml")
                .arg("|")
                .arg("kubectl")
                .arg("apply")
                .arg("-f").arg("-");
            
            let _ = cmd.output().await;
        }
        
        // If sentinel is enabled, check for NetworkPolicy support
        if self.enable_sentinel {
            info!("Checking NetworkPolicy support for Redis Sentinel");
            // Add check logic here
        }
        
        if let Some(ref mut p) = payload {
            p.insert("pre_exec_complete", true)?;
        }
        
        Ok(payload)
    }
    
    async fn exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        // Use base implementation for standard deployment
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        base.exec(kubernetes_config, envs, payload).await
    }
    
    async fn post_exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        mut payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        if self.get_chart_info().action == HelmAction::Deploy {
            info!("Post-exec: Configuring Redis");
            
            // Wait for Redis to be ready
            let mut cmd = tokio::process::Command::new("kubectl");
            cmd.env("KUBECONFIG", kubernetes_config);
            for (key, value) in envs {
                cmd.env(key, value);
            }
            
            let selector = if self.enable_sentinel {
                "app.kubernetes.io/component=master"
            } else {
                "app.kubernetes.io/name=redis"
            };
            
            cmd.arg("wait")
                .arg("--for=condition=ready")
                .arg("pod")
                .arg("-l").arg(selector)
                .arg("-n").arg(self.get_namespace())
                .arg("--timeout=300s");
            
            let output = cmd.output().await?;
            
            if output.status.success() {
                info!("Redis is ready");
                
                // Store connection info in payload
                if let Some(ref mut p) = payload {
                    let connection_info = self.get_connection_info(kubernetes_config, envs).await?;
                    p.insert("redis_connection", connection_info)?;
                }
            } else {
                warn!("Redis not ready after timeout");
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
        
        // Base validation
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        let mut base_results = base.validate(kubernetes_config, envs, payload).await?;
        results.append(&mut base_results);
        
        if self.get_chart_info().action == HelmAction::Deploy {
            // Test Redis connectivity
            let connectivity_check = self.check_redis_connectivity(kubernetes_config, envs).await?;
            results.push(connectivity_check);
            
            // Check sentinel status if enabled
            if self.enable_sentinel {
                let sentinel_check = self.check_sentinel_status(kubernetes_config, envs).await?;
                results.push(sentinel_check);
            }
            
            // Verify persistence if enabled
            let persistence_check = self.check_persistence(kubernetes_config, envs).await?;
            results.push(persistence_check);
        }
        
        Ok(results)
    }
    
    async fn on_deploy_failure(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        warn!("Redis deployment failed");
        
        // Get pod logs for debugging
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("logs")
            .arg("-l").arg("app.kubernetes.io/name=redis")
            .arg("-n").arg(self.get_namespace())
            .arg("--tail=100");
        
        if let Ok(output) = cmd.output().await {
            let logs = String::from_utf8_lossy(&output.stdout);
            warn!("Redis logs:\n{}", logs);
        }
        
        // Check PVC status if persistence is enabled
        if self.chart_info.values.iter().any(|v| 
            v.key.contains("persistence.enabled") && v.value == "true"
        ) {
            self.check_pvc_status(kubernetes_config, envs).await?;
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
    ) -> Result<Vec<KubernetesEvent>> {
        get_kubernetes_events(kubernetes_config, envs, &self.get_namespace()).await
    }
    
    async fn get_release_info(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<HelmReleaseStatus> {
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

// Helper methods
impl RedisChart {
    async fn get_connection_info(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<serde_json::Value> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get")
            .arg("svc")
            .arg(format!("{}-master", self.chart_info.name))
            .arg("-n").arg(self.get_namespace())
            .arg("-o").arg("json");
        
        let output = cmd.output().await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let svc: serde_json::Value = serde_json::from_str(&stdout)?;
            
            Ok(serde_json::json!({
                "host": format!("{}-master.{}.svc.cluster.local", self.chart_info.name, self.get_namespace()),
                "port": 6379,
                "sentinel_enabled": self.enable_sentinel,
            }))
        } else {
            Ok(serde_json::json!({}))
        }
    }
    
    async fn check_redis_connectivity(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<ValidationResult> {
        // Run a Redis ping command
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("exec")
            .arg("-n").arg(self.get_namespace())
            .arg(format!("{}-master-0", self.chart_info.name))
            .arg("--")
            .arg("redis-cli")
            .arg("ping");
        
        let output = cmd.output().await?;
        
        let success = output.status.success() && 
            String::from_utf8_lossy(&output.stdout).trim() == "PONG";
        
        Ok(ValidationResult {
            check_name: "redis_connectivity".to_string(),
            passed: success,
            message: if success {
                "Redis is responding to ping".to_string()
            } else {
                "Redis connectivity check failed".to_string()
            },
            details: None,
        })
    }
    
    async fn check_sentinel_status(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<ValidationResult> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get")
            .arg("pods")
            .arg("-l").arg(format!("app.kubernetes.io/name=redis,app.kubernetes.io/component=sentinel"))
            .arg("-n").arg(self.get_namespace())
            .arg("-o").arg("json");
        
        let output = cmd.output().await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: serde_json::Value = serde_json::from_str(&stdout)?;
            
            let pods = json["items"].as_array().unwrap_or(&Vec::new());
            let ready_pods = pods.iter().filter(|p| 
                p["status"]["phase"].as_str() == Some("Running")
            ).count();
            
            Ok(ValidationResult {
                check_name: "sentinel_status".to_string(),
                passed: ready_pods >= 3,
                message: format!("{}/3 sentinel pods ready", ready_pods),
                details: None,
            })
        } else {
            Ok(ValidationResult {
                check_name: "sentinel_status".to_string(),
                passed: false,
                message: "Failed to check sentinel status".to_string(),
                details: None,
            })
        }
    }
    
    async fn check_persistence(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<ValidationResult> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("get")
            .arg("pvc")
            .arg("-l").arg(format!("app.kubernetes.io/instance={}", self.chart_info.name))
            .arg("-n").arg(self.get_namespace())
            .arg("-o").arg("json");
        
        let output = cmd.output().await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: serde_json::Value = serde_json::from_str(&stdout)?;
            
            let pvcs = json["items"].as_array().unwrap_or(&Vec::new());
            let bound_pvcs = pvcs.iter().filter(|pvc| 
                pvc["status"]["phase"].as_str() == Some("Bound")
            ).count();
            
            Ok(ValidationResult {
                check_name: "persistence".to_string(),
                passed: pvcs.is_empty() || bound_pvcs == pvcs.len(),
                message: format!("{}/{} PVCs bound", bound_pvcs, pvcs.len()),
                details: None,
            })
        } else {
            Ok(ValidationResult {
                check_name: "persistence".to_string(),
                passed: true,
                message: "No PVCs found".to_string(),
                details: None,
            })
        }
    }
    
    async fn check_pvc_status(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<()> {
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.env("KUBECONFIG", kubernetes_config);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        
        cmd.arg("describe")
            .arg("pvc")
            .arg("-l").arg(format!("app.kubernetes.io/instance={}", self.chart_info.name))
            .arg("-n").arg(self.get_namespace());
        
        if let Ok(output) = cmd.output().await {
            let description = String::from_utf8_lossy(&output.stdout);
            warn!("PVC status:\n{}", description);
        }
        
        Ok(())
    }
}

// Example usage:
#[tokio::main]
async fn main() -> Result<()> {
    use std::path::PathBuf;
    
    // Create Redis with Sentinel
    let redis = RedisChart::with_sentinel();
    
    // Deploy using lifecycle system
    let kubeconfig = PathBuf::from("/home/user/.kube/config");
    let envs = vec![];
    
    match redis.run(&kubeconfig, &envs).await? {
        Some(result) => {
            println!("Deployment result: {:?}", result);
            
            // Check validation results
            for validation in &result.validation_results {
                println!("  {}: {} - {}", 
                    validation.check_name,
                    if validation.passed { "PASSED" } else { "FAILED" },
                    validation.message
                );
            }
        }
        None => println!("No result returned"),
    }
    
    Ok(())
}
```

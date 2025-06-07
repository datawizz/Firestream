//! Default lifecycle implementations and utilities

use crate::core::{FirestreamError, Result};
use super::types::*;
use super::trait_def::HelmChart;
use super::executor::*;
use std::path::{Path, PathBuf};
use tracing::{info, warn, debug};
use async_trait::async_trait;

/// Base implementation for common charts
pub struct BaseHelmChart {
    pub chart_info: ChartInfo,
}

#[async_trait]
impl HelmChart for BaseHelmChart {
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
                // Check for breaking version changes
                if let Some(breaking_version) = &chart_info.last_breaking_version_requiring_restart {
                    if self.needs_breaking_change_handling(kubernetes_config, envs, breaking_version).await? {
                        info!("Detected breaking change for {}, performing uninstall first", chart_info.name);
                        
                        // Run pre-upgrade commands
                        for cmd in &breaking_version.pre_upgrade_commands {
                            execute_kubectl_command(kubernetes_config, envs, cmd).await?;
                        }
                        
                        // Uninstall if required
                        if breaking_version.requires_uninstall {
                            helm_uninstall(kubernetes_config, envs, chart_info).await?;
                        }
                    }
                }
                
                // Deploy the chart
                helm_deploy(kubernetes_config, envs, chart_info).await?;
            }
            
            HelmAction::Destroy => {
                if self.is_deployed(kubernetes_config, envs).await? {
                    helm_uninstall(kubernetes_config, envs, chart_info).await?;
                } else {
                    info!("Chart {} is not deployed, skipping uninstall", chart_info.name);
                }
            }
            
            HelmAction::Skip => {
                info!("Skipping chart: {}", chart_info.name);
            }
        }
        
        Ok(payload)
    }
    
    async fn validate(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        _payload: Option<ChartPayload>,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();
        let chart_info = self.get_chart_info();
        
        match chart_info.action {
            HelmAction::Deploy => {
                // Check if release exists
                let release_check = ValidationResult {
                    check_name: "release_exists".to_string(),
                    passed: self.is_deployed(kubernetes_config, envs).await?,
                    message: format!("Helm release '{}' exists", chart_info.name),
                    details: None,
                };
                results.push(release_check);
                
                // Check pods are running
                if let Ok(pod_status) = check_pods_running(
                    kubernetes_config,
                    envs,
                    &self.get_namespace(),
                    &chart_info.name,
                ).await {
                    results.push(pod_status);
                }
            }
            
            HelmAction::Destroy => {
                // Verify removal
                let removal_check = ValidationResult {
                    check_name: "release_removed".to_string(),
                    passed: !self.is_deployed(kubernetes_config, envs).await?,
                    message: format!("Helm release '{}' removed", chart_info.name),
                    details: None,
                };
                results.push(removal_check);
            }
            
            _ => {}
        }
        
        Ok(results)
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
        let chart_info = self.get_chart_info();
        get_helm_release_status(kubernetes_config, envs, &chart_info.name, &self.get_namespace()).await
    }
    
    async fn is_deployed(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<bool> {
        let chart_info = self.get_chart_info();
        is_helm_release_deployed(kubernetes_config, envs, &chart_info.name, &self.get_namespace()).await
    }
    
    async fn check_prerequisites(
        &self,
        mut payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        let chart = self.get_chart_info();
        info!("Checking prerequisites for chart: {}", chart.name);
        
        // Check values files exist and are readable
        for file in &chart.values_files {
            if !file.exists() {
                return Err(FirestreamError::ConfigError(
                    format!("Values file not found: {:?}", file)
                ));
            }
            
            // Try to read the file to ensure permissions
            std::fs::metadata(file)
                .map_err(|e| FirestreamError::ConfigError(
                    format!("Cannot access values file {:?}: {}", file, e)
                ))?;
        }
        
        // Store prerequisites check result in payload
        if let Some(ref mut p) = payload {
            p.insert("prerequisites_checked", true)?;
        }
        
        Ok(payload)
    }
}

impl BaseHelmChart {
    /// Check if breaking change handling is needed
    async fn needs_breaking_change_handling(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        breaking_version: &BreakingVersion,
    ) -> Result<bool> {
        // Get current version if deployed
        if !self.is_deployed(kubernetes_config, envs).await? {
            return Ok(false);
        }
        
        match self.get_release_info(kubernetes_config, envs).await {
            Ok(info) => {
                // Extract version from chart string (e.g., "postgresql-12.1.0" -> "12.1.0")
                if let Some(version) = extract_version_from_chart(&info.chart) {
                    debug!("Current version: {}, breaking version: {}", version, breaking_version.version);
                    Ok(is_version_less_than(&version, &breaking_version.version))
                } else {
                    warn!("Could not extract version from chart: {}", info.chart);
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("Could not get release info: {}", e);
                Ok(false)
            }
        }
    }
}

/// Extract version from chart string (e.g., "postgresql-12.1.0" -> "12.1.0")
fn extract_version_from_chart(chart: &str) -> Option<String> {
    chart.rsplit('-').next().map(|s| s.to_string())
}

/// Compare versions (simple string comparison, could be improved with semver)
fn is_version_less_than(current: &str, target: &str) -> bool {
    // TODO: Use semver crate for proper version comparison
    current < target
}

/// Common chart implementation that uses all defaults
#[derive(Clone)]
pub struct CommonChart {
    pub chart_info: ChartInfo,
}

impl CommonChart {
    pub fn new(chart_info: ChartInfo) -> Self {
        Self { chart_info }
    }
}

#[async_trait]
impl HelmChart for CommonChart {
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
    
    async fn validate(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Vec<ValidationResult>> {
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        base.validate(kubernetes_config, envs, payload).await
    }
    
    async fn collect_events(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<Vec<KubernetesEvent>> {
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        base.collect_events(kubernetes_config, envs).await
    }
    
    async fn get_release_info(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<HelmReleaseStatus> {
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        base.get_release_info(kubernetes_config, envs).await
    }
    
    async fn is_deployed(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<bool> {
        let base = BaseHelmChart {
            chart_info: self.chart_info.clone(),
        };
        base.is_deployed(kubernetes_config, envs).await
    }
}

/// Helper function to create chart path relative to project root
pub fn chart_path(relative_path: &str) -> PathBuf {
    // This could be made configurable
    PathBuf::from("charts").join(relative_path)
}

/// Helper function to create values file path
pub fn values_path(relative_path: &str) -> PathBuf {
    PathBuf::from("values").join(relative_path)
}

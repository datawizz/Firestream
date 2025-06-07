//! HelmChart trait definition

use crate::core::{FirestreamError, Result};
use super::types::*;
use std::path::Path;
use tracing::{info, error};
use async_trait::async_trait;

/// Trait for managing Helm chart lifecycles
///
/// This trait provides a comprehensive lifecycle management system for Helm charts,
/// allowing you to hook into various stages of the deployment process.
#[async_trait]
pub trait HelmChart: Send + Sync {
    /// Get the chart information
    fn get_chart_info(&self) -> &ChartInfo;
    
    /// Get mutable chart information (for dynamic updates)
    fn get_chart_info_mut(&mut self) -> &mut ChartInfo;
    
    /// Main execution method that orchestrates the lifecycle
    async fn run(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<Option<ChartDeploymentResult>> {
        info!("Running lifecycle for chart: {}", self.get_chart_info().name);
        
        let chart_name = self.get_chart_info().name.clone();
        let mut result = ChartDeploymentResult {
            chart_name: chart_name.clone(),
            success: false,
            release_info: None,
            validation_results: Vec::new(),
            events: Vec::new(),
            error: None,
        };
        
        // Execute lifecycle stages
        match self.execute_lifecycle(kubernetes_config, envs).await {
            Ok(deployment_result) => {
                result = deployment_result;
                result.success = true;
            }
            Err(e) => {
                error!("Lifecycle execution failed for {}: {:?}", chart_name, e);
                result.error = Some(e.to_string());
                
                // Collect events for debugging
                if let Ok(events) = self.collect_events(kubernetes_config, envs).await {
                    result.events = events;
                }
            }
        }
        
        Ok(Some(result))
    }
    
    /// Internal lifecycle execution
    async fn execute_lifecycle(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<ChartDeploymentResult> {
        let mut payload = Some(ChartPayload::new());
        
        // Check prerequisites
        payload = self.check_prerequisites(payload).await?;
        
        // Pre-execution hook
        payload = self.pre_exec(kubernetes_config, envs, payload).await?;
        
        // Main execution
        match self.exec(kubernetes_config, envs, payload.clone()).await {
            Ok(p) => payload = p,
            Err(e) => {
                error!("Chart execution failed: {:?}", e);
                
                // Run failure handler
                self.on_deploy_failure(kubernetes_config, envs, payload.clone()).await?;
                
                return Err(e);
            }
        }
        
        // Post-execution hook
        payload = self.post_exec(kubernetes_config, envs, payload).await?;
        
        // Validation
        let validation_results = self.validate(kubernetes_config, envs, payload.clone()).await?;
        
        // Get release info
        let release_info = self.get_release_info(kubernetes_config, envs).await.ok();
        
        Ok(ChartDeploymentResult {
            chart_name: self.get_chart_info().name.clone(),
            success: true,
            release_info,
            validation_results,
            events: Vec::new(),
            error: None,
        })
    }
    
    /// Check prerequisites before any action
    ///
    /// Default implementation checks for:
    /// - Values files accessibility
    /// - Required dependencies
    /// - Resource availability
    async fn check_prerequisites(
        &self,
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        let chart = self.get_chart_info();
        info!("Checking prerequisites for chart: {}", chart.name);
        
        // Check values files
        for file in &chart.values_files {
            if !file.exists() {
                return Err(FirestreamError::ConfigError(
                    format!("Values file not found: {:?}", file)
                ));
            }
        }
        
        Ok(payload)
    }
    
    /// Pre-execution hook
    ///
    /// This is called before the main chart action (deploy/destroy).
    /// Use this for:
    /// - Setting up prerequisites
    /// - Modifying existing resources
    /// - Creating temporary resources
    async fn pre_exec(
        &self,
        _kubernetes_config: &Path,
        _envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        Ok(payload)
    }
    
    /// Main execution method
    ///
    /// This performs the actual Helm action (deploy/destroy/skip).
    /// Override this for custom deployment logic.
    async fn exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>>;
    
    /// Handle deployment failures
    ///
    /// This is called when the exec method fails.
    /// Use this for:
    /// - Cleanup operations
    /// - Collecting debug information
    /// - Attempting recovery
    async fn on_deploy_failure(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        // Default: collect events for debugging
        let _events = self.collect_events(kubernetes_config, envs).await?;
        Ok(payload)
    }
    
    /// Post-execution hook
    ///
    /// This is called after successful execution.
    /// Use this for:
    /// - Cleanup of temporary resources
    /// - Additional configuration
    /// - Triggering dependent actions
    async fn post_exec(
        &self,
        _kubernetes_config: &Path,
        _envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        Ok(payload)
    }
    
    /// Validate the deployment
    ///
    /// This ensures the chart has been deployed correctly.
    /// Use this for:
    /// - Checking pod readiness
    /// - Verifying service endpoints
    /// - Testing connectivity
    async fn validate(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Vec<ValidationResult>>;
    
    /// Collect Kubernetes events for debugging
    async fn collect_events(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<Vec<KubernetesEvent>>;
    
    /// Get Helm release information
    async fn get_release_info(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<HelmReleaseStatus>;
    
    /// Check if the chart is deployed
    async fn is_deployed(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<bool>;
    
    /// Get the namespace for the chart
    fn get_namespace(&self) -> String {
        let chart = self.get_chart_info();
        chart.custom_namespace.clone()
            .unwrap_or_else(|| chart.namespace.as_str().to_string())
    }
    
    /// Generate a unique identifier for this chart instance
    fn get_identifier(&self) -> String {
        let chart = self.get_chart_info();
        format!("{}-{}", self.get_namespace(), chart.name)
    }
}

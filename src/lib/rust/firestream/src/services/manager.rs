//! Service manager implementation
//!
//! This module provides the core service management functionality.

use crate::config::{ConfigManager, ServiceConfig, ServiceState, ServiceStatus, ResourceUsage};
use crate::core::{FirestreamError, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Service manager for handling service operations
pub struct ServiceManager {
    config_manager: ConfigManager,
    kube_client: Option<kube::Client>,
}

impl ServiceManager {
    /// Create a new service manager
    pub async fn new() -> anyhow::Result<Self> {
        let config_manager = ConfigManager::new()?;
        
        // Try to create Kubernetes client
        let kube_client = match kube::Client::try_default().await {
            Ok(client) => {
                info!("Successfully connected to Kubernetes cluster");
                Some(client)
            }
            Err(e) => {
                warn!("Failed to connect to Kubernetes cluster: {}", e);
                warn!("Running in offline mode - some features will be limited");
                None
            }
        };

        Ok(Self {
            config_manager,
            kube_client,
        })
    }

    /// Install a service
    pub async fn install_service(&self, service_name: &str, config_path: Option<&str>) -> Result<()> {
        info!("Installing service: {}", service_name);

        // Load or create service configuration
        let config = if let Some(path) = config_path {
            // Load from custom path
            let content = tokio::fs::read_to_string(path)
                .await
                .map_err(|e| FirestreamError::ConfigError(format!("Failed to read config file: {}", e)))?;
            
            toml::from_str::<ServiceConfig>(&content)
                .map_err(|e| FirestreamError::ConfigError(format!("Failed to parse config: {}", e)))?
        } else {
            // Try to load existing config or create default
            match self.config_manager.load_service_config(service_name).await {
                Ok(config) => config,
                Err(_) => {
                    return Err(FirestreamError::ServiceNotFound(
                        format!("No configuration found for service: {}", service_name)
                    ));
                }
            }
        };

        // Validate configuration
        self.validate_service_config(&config)?;

        // Check dependencies
        self.check_dependencies(&config).await?;

        // Verify resource availability
        self.verify_resources(&config)?;

        // Deploy the service
        self.deploy_service(service_name, &config).await?;

        // Update service state
        let mut states = self.config_manager.load_service_states().await
            .unwrap_or_default();
        
        states.insert(service_name.to_string(), ServiceState {
            name: service_name.to_string(),
            status: ServiceStatus::Running,
            installed_version: Some(config.metadata.version.clone()),
            last_updated: chrono::Utc::now(),
            resource_usage: None,
        });

        self.config_manager.save_service_states(&states).await
            .map_err(|e| FirestreamError::GeneralError(format!("Failed to save state: {}", e)))?;

        info!("Service {} installed successfully", service_name);
        Ok(())
    }

    /// Start a service
    pub async fn start_service(&self, service_name: &str) -> Result<()> {
        info!("Starting service: {}", service_name);

        let mut states = self.config_manager.load_service_states().await
            .unwrap_or_default();

        let state = states.get_mut(service_name)
            .ok_or_else(|| FirestreamError::ServiceNotFound(service_name.to_string()))?;

        if state.status == ServiceStatus::Running {
            return Err(FirestreamError::GeneralError("Service is already running".to_string()));
        }

        // Start the service using Kubernetes API
        if let Some(_client) = &self.kube_client {
            // TODO: Implement actual Kubernetes deployment scaling
            debug!("Scaling deployment for service: {}", service_name);
        }

        state.status = ServiceStatus::Running;
        state.last_updated = chrono::Utc::now();

        self.config_manager.save_service_states(&states).await
            .map_err(|e| FirestreamError::GeneralError(format!("Failed to save state: {}", e)))?;

        info!("Service {} started successfully", service_name);
        Ok(())
    }

    /// Stop a service
    pub async fn stop_service(&self, service_name: &str) -> Result<()> {
        info!("Stopping service: {}", service_name);

        let mut states = self.config_manager.load_service_states().await
            .unwrap_or_default();

        let state = states.get_mut(service_name)
            .ok_or_else(|| FirestreamError::ServiceNotFound(service_name.to_string()))?;

        if state.status == ServiceStatus::Stopped {
            return Err(FirestreamError::GeneralError("Service is already stopped".to_string()));
        }

        // Stop the service using Kubernetes API
        if let Some(_client) = &self.kube_client {
            // TODO: Implement actual Kubernetes deployment scaling to 0
            debug!("Scaling down deployment for service: {}", service_name);
        }

        state.status = ServiceStatus::Stopped;
        state.last_updated = chrono::Utc::now();

        self.config_manager.save_service_states(&states).await
            .map_err(|e| FirestreamError::GeneralError(format!("Failed to save state: {}", e)))?;

        info!("Service {} stopped successfully", service_name);
        Ok(())
    }

    /// Get status of all services or a specific service
    pub async fn get_status(&self, service_name: Option<&str>) -> Result<HashMap<String, ServiceState>> {
        let states = self.config_manager.load_service_states().await
            .unwrap_or_default();

        if let Some(name) = service_name {
            if let Some(state) = states.get(name) {
                let mut result = HashMap::new();
                result.insert(name.to_string(), state.clone());
                Ok(result)
            } else {
                Err(FirestreamError::ServiceNotFound(name.to_string()))
            }
        } else {
            Ok(states)
        }
    }

    /// List available services
    pub async fn list_available_services(&self) -> Result<Vec<String>> {
        // TODO: Scan deployment packages directory
        // For now, return a hardcoded list
        Ok(vec![
            "kafka".to_string(),
            "postgresql".to_string(),
            "airflow".to_string(),
            "redis".to_string(),
            "elasticsearch".to_string(),
        ])
    }

    /// Validate service configuration
    fn validate_service_config(&self, config: &ServiceConfig) -> Result<()> {
        // Basic validation
        if config.metadata.name.is_empty() {
            return Err(FirestreamError::ConfigError("Service name cannot be empty".to_string()));
        }

        if config.metadata.version.is_empty() {
            return Err(FirestreamError::ConfigError("Service version cannot be empty".to_string()));
        }

        Ok(())
    }

    /// Check service dependencies
    async fn check_dependencies(&self, config: &ServiceConfig) -> Result<()> {
        let states = self.config_manager.load_service_states().await
            .unwrap_or_default();

        for dep in &config.spec.dependencies {
            if let Some(state) = states.get(dep) {
                if state.status != ServiceStatus::Running {
                    return Err(FirestreamError::DependencyError(
                        format!("Dependency {} is not running", dep)
                    ));
                }
            } else {
                return Err(FirestreamError::DependencyError(
                    format!("Dependency {} is not installed", dep)
                ));
            }
        }

        Ok(())
    }

    /// Verify resource availability
    fn verify_resources(&self, _config: &ServiceConfig) -> Result<()> {
        // TODO: Implement actual resource checking
        // For MVP, just return OK
        Ok(())
    }

    /// Deploy a service
    async fn deploy_service(&self, service_name: &str, _config: &ServiceConfig) -> Result<()> {
        if let Some(_client) = &self.kube_client {
            // TODO: Implement actual Kubernetes deployment
            debug!("Deploying service {} to Kubernetes", service_name);
        } else {
            warn!("Running in offline mode - skipping actual deployment");
        }

        Ok(())
    }

    /// Get service logs
    pub async fn get_logs(&self, _service_name: &str, _follow: bool) -> Result<Vec<String>> {
        // TODO: Implement log streaming from Kubernetes
        Ok(vec![
            format!("[{}] Service started", chrono::Utc::now()),
            format!("[{}] Health check passed", chrono::Utc::now()),
        ])
    }

    /// Get resource usage for services
    pub async fn get_resource_usage(&self, service_name: Option<&str>) -> Result<HashMap<String, ResourceUsage>> {
        // TODO: Implement actual resource monitoring
        let mut usage = HashMap::new();
        
        if let Some(name) = service_name {
            usage.insert(name.to_string(), ResourceUsage {
                cpu_cores: 0.5,
                memory_mb: 1024,
                disk_gb: 10,
            });
        }

        Ok(usage)
    }
}

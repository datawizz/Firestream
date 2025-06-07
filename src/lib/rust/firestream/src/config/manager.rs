//! Configuration manager for handling config files
//!
//! This module provides functionality to load, save, and manage configuration files.

use super::schema::*;
use super::config_dir;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Configuration manager for handling all config operations
pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            config_dir: config_dir()?,
        })
    }

    /// Load the global configuration
    pub async fn load_global_config(&self) -> Result<GlobalConfig> {
        let config_path = self.config_dir.join("config.toml");
        
        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(GlobalConfig::default());
        }

        let content = fs::read_to_string(&config_path)
            .await
            .context("Failed to read global config file")?;
        
        toml::from_str(&content)
            .context("Failed to parse global config")
    }

    /// Save the global configuration
    pub async fn save_global_config(&self, config: &GlobalConfig) -> Result<()> {
        let config_path = self.config_dir.join("config.toml");
        let content = toml::to_string_pretty(config)
            .context("Failed to serialize global config")?;
        
        fs::write(&config_path, content)
            .await
            .context("Failed to write global config file")?;
        
        Ok(())
    }

    /// Load a service configuration
    pub async fn load_service_config(&self, service_name: &str) -> Result<ServiceConfig> {
        let config_path = self.config_dir
            .join("services")
            .join(format!("{}.toml", service_name));
        
        let content = fs::read_to_string(&config_path)
            .await
            .context(format!("Failed to read config for service: {}", service_name))?;
        
        toml::from_str(&content)
            .context(format!("Failed to parse config for service: {}", service_name))
    }

    /// Save a service configuration
    pub async fn save_service_config(&self, service_name: &str, config: &ServiceConfig) -> Result<()> {
        let config_path = self.config_dir
            .join("services")
            .join(format!("{}.toml", service_name));
        
        let content = toml::to_string_pretty(config)
            .context("Failed to serialize service config")?;
        
        fs::write(&config_path, content)
            .await
            .context(format!("Failed to write config for service: {}", service_name))?;
        
        Ok(())
    }

    /// List all available service configurations
    pub async fn list_service_configs(&self) -> Result<Vec<String>> {
        let services_dir = self.config_dir.join("services");
        
        if !services_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&services_dir).await?;
        let mut services = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "toml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    services.push(name.to_string());
                }
            }
        }

        Ok(services)
    }

    /// Load service state
    pub async fn load_service_states(&self) -> Result<HashMap<String, ServiceState>> {
        let state_path = self.config_dir.join("state").join("services.json");
        
        if !state_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&state_path)
            .await
            .context("Failed to read service state file")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse service state")
    }

    /// Save service state
    pub async fn save_service_states(&self, states: &HashMap<String, ServiceState>) -> Result<()> {
        let state_path = self.config_dir.join("state").join("services.json");
        let content = serde_json::to_string_pretty(states)
            .context("Failed to serialize service state")?;
        
        fs::write(&state_path, content)
            .await
            .context("Failed to write service state file")?;
        
        Ok(())
    }

    /// Check if a service configuration exists
    pub async fn service_config_exists(&self, service_name: &str) -> bool {
        let config_path = self.config_dir
            .join("services")
            .join(format!("{}.toml", service_name));
        
        config_path.exists()
    }

    /// Delete a service configuration
    pub async fn delete_service_config(&self, service_name: &str) -> Result<()> {
        let config_path = self.config_dir
            .join("services")
            .join(format!("{}.toml", service_name));
        
        if config_path.exists() {
            fs::remove_file(&config_path)
                .await
                .context(format!("Failed to delete config for service: {}", service_name))?;
        }
        
        Ok(())
    }

    /// Get the configuration directory path
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }
}

use std::collections::HashMap;

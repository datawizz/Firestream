//! Configuration module for Firestream
//! 
//! This module handles all configuration-related functionality including
//! global and service-specific configurations.

pub mod schema;
pub mod manager;

pub use schema::{GlobalConfig, ServiceConfig, ServiceMetadata, ServiceSpec, ServiceState, ServiceStatus, ResourceUsage};
pub use manager::ConfigManager;

use std::path::PathBuf;
use anyhow::Result;

/// Get the default configuration directory path
pub fn config_dir() -> Result<PathBuf> {
    let base_dirs = directories::BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    
    Ok(base_dirs.home_dir().join(".firestream"))
}

/// Initialize the configuration directory structure
pub async fn init_config_dir() -> Result<()> {
    let config_path = config_dir()?;
    
    // Create main config directory
    tokio::fs::create_dir_all(&config_path).await?;
    
    // Create subdirectories
    tokio::fs::create_dir_all(config_path.join("services")).await?;
    tokio::fs::create_dir_all(config_path.join("state")).await?;
    
    Ok(())
}

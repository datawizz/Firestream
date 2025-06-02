//! Firestream app deployment module
//!
//! This module provides an opinionated approach to deploying applications
//! in Firestream using:
//! - firestream.toml manifests (similar to Cargo.toml)
//! - bootstrap.sh as the standard entrypoint
//! - Dockerfile for container image building
//! - Dependency management between apps

pub mod schema;
pub mod manifest;
pub mod lifecycle;
pub mod builder;
pub mod bootstrap;

pub use schema::{AppManifest, AppMetadata, AppDependency, AppConfig};
pub use manifest::ManifestParser;
pub use lifecycle::AppLifecycle;
pub use builder::AppBuilder;
pub use bootstrap::BootstrapExecutor;

// Re-export common types
pub use super::{HelmChart, ChartInfo, HelmAction};

/// Standard app directory structure
pub struct AppStructure {
    /// Root directory of the app
    pub root: std::path::PathBuf,
    /// Path to firestream.toml
    pub manifest: std::path::PathBuf,
    /// Path to bootstrap.sh
    pub bootstrap: std::path::PathBuf,
    /// Path to Dockerfile
    pub dockerfile: std::path::PathBuf,
    /// Path to values.yaml (optional)
    pub values: Option<std::path::PathBuf>,
    /// Path to Chart.yaml (optional)
    pub chart: Option<std::path::PathBuf>,
}

impl AppStructure {
    /// Create from app root directory
    pub fn from_root(root: impl Into<std::path::PathBuf>) -> Self {
        let root = root.into();
        Self {
            manifest: root.join("firestream.toml"),
            bootstrap: root.join("bootstrap.sh"),
            dockerfile: root.join("Dockerfile"),
            values: Some(root.join("values.yaml")).filter(|p| p.exists()),
            chart: Some(root.join("Chart.yaml")).filter(|p| p.exists()),
            root,
        }
    }
    
    /// Validate that required files exist
    pub fn validate(&self) -> crate::core::Result<()> {
        use crate::core::FirestreamError;
        
        if !self.manifest.exists() {
            return Err(FirestreamError::ConfigError(
                format!("firestream.toml not found at {:?}", self.manifest)
            ));
        }
        
        if !self.bootstrap.exists() {
            return Err(FirestreamError::ConfigError(
                format!("bootstrap.sh not found at {:?}", self.bootstrap)
            ));
        }
        
        if !self.dockerfile.exists() {
            return Err(FirestreamError::ConfigError(
                format!("Dockerfile not found at {:?}", self.dockerfile)
            ));
        }
        
        Ok(())
    }
}

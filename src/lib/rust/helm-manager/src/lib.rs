//! Helm Manager - A Rust library for managing Helm charts with embedded Bitnami charts
//!
//! This library provides a functional interface for deploying and managing Helm releases
//! with a focus on Bitnami charts. It embeds the charts at compile time for deterministic
//! deployments.
//!
//! # Example
//!
//! ```no_run
//! use helm_manager::{HelmManager, Deployment};
//!
//! # async fn example() -> Result<(), helm_manager::Error> {
//! let manager = HelmManager::new().await?;
//!
//! let deployment = Deployment::builder("postgresql")
//!     .name("my-database")
//!     .namespace("default")
//!     .env_file("/workspace/etc/.env")
//!     .build()?;
//!
//! let release = manager.deploy(deployment).await?;
//! # Ok(())
//! # }
//! ```

pub mod chart_manager;
pub mod config;
pub mod deployment;
pub mod embedded_charts;
pub mod error;
pub mod helm_client;
pub mod kubectl_client;
pub mod models;
pub mod providers;
pub mod traits;
pub mod values_resolver;

// Re-export commonly used types
pub use config::Config;
pub use deployment::{Deployment, DeploymentBuilder, Stack, StackBuilder};
pub use error::{Error, Result};
pub use models::{Chart, ChartMetadata, Release, ReleaseStatus, Values};
pub use traits::HelmManagerTrait;

use std::sync::Arc;

/// The main HelmManager struct that provides chart lifecycle management
pub struct HelmManager {
    #[allow(dead_code)]
    config: Arc<Config>,
    helm_client: Arc<helm_client::HelmClient>,
    #[allow(dead_code)]
    kubectl_client: Arc<kubectl_client::KubectlClient>,
    chart_manager: Arc<chart_manager::ChartManager>,
}

impl HelmManager {
    /// Create a new HelmManager instance
    pub async fn new() -> Result<Self> {
        Self::with_config(Config::default()).await
    }

    /// Create a new HelmManager instance with custom configuration
    pub async fn with_config(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        let helm_client = Arc::new(helm_client::HelmClient::new()?);
        let kubectl_client = Arc::new(kubectl_client::KubectlClient::new()?);
        let chart_manager = Arc::new(chart_manager::ChartManager::new());

        Ok(Self {
            config,
            helm_client,
            kubectl_client,
            chart_manager,
        })
    }

    /// Deploy a single chart
    pub async fn deploy(&self, deployment: Deployment) -> Result<Release> {
        self.deploy_impl(deployment).await
    }

    /// Deploy a stack of related charts
    pub async fn deploy_stack(&self, stack: Stack) -> Result<Vec<Release>> {
        let mut releases = Vec::new();
        
        // TODO: Implement dependency ordering
        for deployment in stack.deployments {
            let release = self.deploy_impl(deployment).await?;
            releases.push(release);
        }
        
        Ok(releases)
    }

    /// Update an existing release
    pub async fn update(&self, release_name: &str, deployment: Deployment) -> Result<Release> {
        self.update_impl(release_name, deployment).await
    }

    /// Rollback a release to a previous revision
    pub async fn rollback(&self, release_name: &str, revision: u32) -> Result<()> {
        self.helm_client.rollback(release_name, revision).await
    }

    /// Delete a release
    pub async fn delete(&self, release_name: &str) -> Result<()> {
        self.helm_client.uninstall(release_name).await
    }

    /// Get the status of a release
    pub async fn status(&self, release_name: &str) -> Result<ReleaseStatus> {
        self.helm_client.status(release_name).await
    }

    /// List all releases
    pub async fn list_releases(&self) -> Result<Vec<Release>> {
        self.helm_client.list().await
    }

    /// List available charts
    pub fn list_charts(&self) -> Vec<&str> {
        self.chart_manager.list_charts()
    }

    // Private implementation methods
    async fn deploy_impl(&self, deployment: Deployment) -> Result<Release> {
        // Extract chart if needed
        let chart_path = self.chart_manager.get_chart_path(&deployment.chart).await?;
        
        // Resolve values
        let values = values_resolver::resolve_values(&deployment).await?;
        
        // Deploy using helm
        self.helm_client.install(
            &deployment.name,
            &chart_path,
            &deployment.namespace,
            values,
            deployment.wait,
            deployment.atomic,
        ).await
    }

    async fn update_impl(&self, release_name: &str, deployment: Deployment) -> Result<Release> {
        // Extract chart if needed
        let chart_path = self.chart_manager.get_chart_path(&deployment.chart).await?;
        
        // Resolve values
        let values = values_resolver::resolve_values(&deployment).await?;
        
        // Update using helm
        self.helm_client.upgrade(
            release_name,
            &chart_path,
            &deployment.namespace,
            values,
            deployment.wait,
            deployment.atomic,
        ).await
    }
}
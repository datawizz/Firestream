use async_trait::async_trait;
use crate::{Deployment, Release, ReleaseStatus, Result, Stack};

/// Core trait for Helm lifecycle management
#[async_trait]
pub trait HelmManagerTrait: Send + Sync {
    /// Deploy a single chart
    async fn deploy(&self, deployment: Deployment) -> Result<Release>;

    /// Deploy a stack of related charts
    async fn deploy_stack(&self, stack: Stack) -> Result<Vec<Release>>;

    /// Update an existing release
    async fn update(&self, release_name: &str, deployment: Deployment) -> Result<Release>;

    /// Rollback a release to a previous revision
    async fn rollback(&self, release_name: &str, revision: u32) -> Result<()>;

    /// Delete a release
    async fn delete(&self, release_name: &str) -> Result<()>;

    /// Get the status of a release
    async fn status(&self, release_name: &str) -> Result<ReleaseStatus>;

    /// List all releases
    async fn list_releases(&self) -> Result<Vec<Release>>;

    /// List available charts
    fn list_charts(&self) -> Vec<&str>;
}

/// Trait for chart providers
#[async_trait]
pub trait ChartProvider: Send + Sync {
    /// Get the path to a chart
    async fn get_chart_path(&self, chart_name: &str) -> Result<String>;

    /// List available charts
    fn list_charts(&self) -> Vec<&str>;

    /// Get chart metadata
    async fn get_chart_metadata(&self, chart_name: &str) -> Result<crate::ChartMetadata>;

    /// Get default values for a chart
    async fn get_default_values(&self, chart_name: &str) -> Result<serde_json::Value>;
}

/// Trait for values providers
#[async_trait]
pub trait ValuesProvider: Send + Sync {
    /// Resolve values for a deployment
    async fn resolve_values(&self, deployment: &Deployment) -> Result<serde_json::Value>;
}

/// Trait for Helm client operations
#[async_trait]
pub trait HelmClientTrait: Send + Sync {
    /// Install a new release
    async fn install(
        &self,
        name: &str,
        chart: &str,
        namespace: &str,
        values: serde_json::Value,
        wait: bool,
        atomic: bool,
    ) -> Result<Release>;

    /// Upgrade an existing release
    async fn upgrade(
        &self,
        name: &str,
        chart: &str,
        namespace: &str,
        values: serde_json::Value,
        wait: bool,
        atomic: bool,
    ) -> Result<Release>;

    /// Uninstall a release
    async fn uninstall(&self, name: &str) -> Result<()>;

    /// Rollback a release
    async fn rollback(&self, name: &str, revision: u32) -> Result<()>;

    /// Get release status
    async fn status(&self, name: &str) -> Result<ReleaseStatus>;

    /// List releases
    async fn list(&self) -> Result<Vec<Release>>;
}

/// Trait for kubectl client operations
#[async_trait]
pub trait KubectlClientTrait: Send + Sync {
    /// Check if a namespace exists
    async fn namespace_exists(&self, name: &str) -> Result<bool>;

    /// Create a namespace
    async fn create_namespace(&self, name: &str) -> Result<()>;

    /// Get pods for a release
    async fn get_pods(&self, namespace: &str, selector: &str) -> Result<Vec<String>>;

    /// Wait for pods to be ready
    async fn wait_for_ready(&self, namespace: &str, selector: &str, timeout: u64) -> Result<()>;
}
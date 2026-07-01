use async_trait::async_trait;
use crate::{Deployment, Release, ReleaseStatus, Result};

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

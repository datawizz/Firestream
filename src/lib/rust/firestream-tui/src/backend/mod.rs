// Backend API clients with placeholders

pub mod api_client;
pub mod mock_client;

pub use api_client::ApiClient;
pub use mock_client::MockClient;

use std::future::Future;
use std::pin::Pin;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Server error: {message}")]
    Server { message: String },
    
    #[error("Authentication error")]
    Authentication,
    
    #[error("Not found")]
    NotFound,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait FirestreamBackend: Send + Sync {
    // Template operations
    fn list_templates(&self) -> BoxFuture<'_, ApiResult<Vec<crate::models::Template>>>;
    fn get_template(&self, id: &str) -> BoxFuture<'_, ApiResult<crate::models::Template>>;
    
    // Deployment operations
    fn list_deployments(&self) -> BoxFuture<'_, ApiResult<Vec<crate::models::Deployment>>>;
    fn get_deployment(&self, id: &str) -> BoxFuture<'_, ApiResult<crate::models::DeploymentDetail>>;
    fn create_deployment(&self, template_id: &str, name: &str, payload: serde_json::Value) -> BoxFuture<'_, ApiResult<crate::models::Deployment>>;
    fn scale_deployment(&self, id: &str, replicas: u32) -> BoxFuture<'_, ApiResult<crate::models::Deployment>>;
    fn delete_deployment(&self, id: &str) -> BoxFuture<'_, ApiResult<()>>;
    fn get_deployment_logs(&self, id: &str, tail: Option<u32>) -> BoxFuture<'_, ApiResult<String>>;
    
    // Cluster operations
    fn get_cluster_status(&self) -> BoxFuture<'_, ApiResult<crate::models::ClusterStatus>>;
    fn list_services(&self) -> BoxFuture<'_, ApiResult<Vec<crate::models::Service>>>;
    
    // Node operations
    fn list_nodes(&self) -> BoxFuture<'_, ApiResult<Vec<crate::models::Node>>>;
    fn get_node(&self, id: &str) -> BoxFuture<'_, ApiResult<crate::models::NodeDetail>>;
    fn provision_node(&self, provider: &str, instance_type: &str) -> BoxFuture<'_, ApiResult<String>>;
    
    // Data operations
    fn list_delta_tables(&self) -> BoxFuture<'_, ApiResult<Vec<crate::models::DeltaTable>>>;
    fn get_table_schema(&self, table_name: &str) -> BoxFuture<'_, ApiResult<crate::models::TableSchema>>;
    fn list_lakefs_branches(&self) -> BoxFuture<'_, ApiResult<Vec<crate::models::LakeFSBranch>>>;
    fn list_s3_buckets(&self) -> BoxFuture<'_, ApiResult<Vec<crate::models::S3Bucket>>>;
    
    // Build operations
    fn start_build(&self, deployment_id: &str) -> BoxFuture<'_, ApiResult<crate::models::BuildStatus>>;
    fn get_build_status(&self, build_id: &str) -> BoxFuture<'_, ApiResult<crate::models::BuildStatus>>;
    fn get_build_logs(&self, build_id: &str) -> BoxFuture<'_, ApiResult<String>>;
    
    // Secret operations
    fn list_secrets(&self) -> BoxFuture<'_, ApiResult<Vec<crate::models::SecretInfo>>>;
    fn create_secret(&self, name: &str, data: std::collections::HashMap<String, String>) -> BoxFuture<'_, ApiResult<crate::models::SecretInfo>>;
    fn delete_secret(&self, name: &str) -> BoxFuture<'_, ApiResult<()>>;
}

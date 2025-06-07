//! Firestream app manifest schema (firestream.toml)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use semver::VersionReq;

/// Main app manifest structure (firestream.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    /// App metadata
    pub app: AppMetadata,
    
    /// App configuration
    #[serde(default)]
    pub config: AppConfig,
    
    /// App dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, AppDependency>,
    
    /// Environment-specific configurations
    #[serde(default)]
    pub environments: HashMap<String, EnvironmentConfig>,
    
    /// Secrets configuration
    #[serde(default)]
    pub secrets: SecretsConfig,
    
    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,
    
    /// Deploy configuration
    #[serde(default)]
    pub deploy: DeployConfig,
}

/// App metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    /// App name (must be unique in the cluster)
    pub name: String,
    
    /// App version (semver)
    pub version: String,
    
    /// App description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// App authors
    #[serde(default)]
    pub authors: Vec<String>,
    
    /// App license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    
    /// App homepage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    
    /// App repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    
    /// App type (service, job, database, etc.)
    #[serde(default = "default_app_type")]
    pub app_type: AppType,
}

/// App type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AppType {
    /// Long-running service
    Service,
    /// Batch job
    Job,
    /// CronJob
    CronJob,
    /// Database
    Database,
    /// Message queue
    MessageQueue,
    /// Infrastructure component
    Infrastructure,
    /// Custom type
    Custom(String),
}

fn default_app_type() -> AppType {
    AppType::Service
}

/// App dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AppDependency {
    /// Simple version string
    Version(String),
    /// Detailed dependency
    Detailed {
        /// Version requirement (semver)
        version: String,
        /// Optional features to enable
        #[serde(default)]
        features: Vec<String>,
        /// Whether this is optional
        #[serde(default)]
        optional: bool,
        /// Override default namespace
        #[serde(skip_serializing_if = "Option::is_none")]
        namespace: Option<String>,
        /// Wait for dependency to be ready
        #[serde(default = "default_wait")]
        wait: bool,
    },
}

fn default_wait() -> bool {
    true
}

impl AppDependency {
    /// Get version requirement
    pub fn version_req(&self) -> crate::core::Result<VersionReq> {
        let version_str = match self {
            AppDependency::Version(v) => v,
            AppDependency::Detailed { version, .. } => version,
        };
        
        VersionReq::parse(version_str)
            .map_err(|e| crate::core::FirestreamError::ConfigError(
                format!("Invalid version requirement: {}", e)
            ))
    }
    
    /// Check if dependency should wait
    pub fn should_wait(&self) -> bool {
        match self {
            AppDependency::Version(_) => true,
            AppDependency::Detailed { wait, .. } => *wait,
        }
    }
}

/// App configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    /// Port configurations
    #[serde(default)]
    pub ports: Vec<PortConfig>,
    
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    
    /// Resource requirements
    #[serde(default)]
    pub resources: ResourceConfig,
    
    /// Volume mounts
    #[serde(default)]
    pub volumes: Vec<VolumeConfig>,
    
    /// Health checks
    #[serde(default)]
    pub health: HealthConfig,
    
    /// Scaling configuration
    #[serde(default)]
    pub scaling: ScalingConfig,
}

/// Port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// Port name
    pub name: String,
    /// Container port
    pub port: u16,
    /// Service port (defaults to container port)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_port: Option<u16>,
    /// Protocol (TCP/UDP)
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_protocol() -> String {
    "TCP".to_string()
}

/// Resource configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// CPU request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_request: Option<String>,
    /// CPU limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_limit: Option<String>,
    /// Memory request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_request: Option<String>,
    /// Memory limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<String>,
}

/// Volume configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    /// Volume name
    pub name: String,
    /// Mount path in container
    pub mount_path: String,
    /// Volume type
    #[serde(flatten)]
    pub volume_type: VolumeType,
}

/// Volume type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VolumeType {
    /// Empty directory
    EmptyDir { 
        #[serde(skip_serializing_if = "Option::is_none")]
        size_limit: Option<String> 
    },
    /// Host path
    HostPath { path: String },
    /// Persistent volume claim
    Pvc { claim_name: String },
    /// ConfigMap
    ConfigMap { name: String },
    /// Secret
    Secret { name: String },
}

/// Health check configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Liveness probe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liveness: Option<ProbeConfig>,
    /// Readiness probe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readiness: Option<ProbeConfig>,
    /// Startup probe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup: Option<ProbeConfig>,
}

/// Probe configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeConfig {
    /// Probe type
    #[serde(flatten)]
    pub probe_type: ProbeType,
    /// Initial delay seconds
    #[serde(default = "default_initial_delay")]
    pub initial_delay_seconds: u32,
    /// Period seconds
    #[serde(default = "default_period")]
    pub period_seconds: u32,
    /// Timeout seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
    /// Success threshold
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,
    /// Failure threshold
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
}

fn default_initial_delay() -> u32 { 0 }
fn default_period() -> u32 { 10 }
fn default_timeout() -> u32 { 1 }
fn default_success_threshold() -> u32 { 1 }
fn default_failure_threshold() -> u32 { 3 }

/// Probe type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProbeType {
    /// HTTP GET probe
    Http { 
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        port: Option<u16>,
    },
    /// TCP socket probe
    Tcp { port: u16 },
    /// Exec probe
    Exec { command: Vec<String> },
}

/// Scaling configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Number of replicas
    #[serde(default = "default_replicas")]
    pub replicas: u32,
    /// Min replicas for autoscaling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_replicas: Option<u32>,
    /// Max replicas for autoscaling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_replicas: Option<u32>,
    /// Target CPU utilization for autoscaling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cpu_utilization: Option<u32>,
}

fn default_replicas() -> u32 { 1 }

/// Environment-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Override app config for this environment
    #[serde(flatten)]
    pub config: AppConfig,
    /// Environment-specific secrets
    #[serde(default)]
    pub secrets: SecretsConfig,
}

/// Secrets configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Inline secrets (for development only!)
    #[serde(default)]
    pub inline: HashMap<String, String>,
    /// Reference to Kubernetes secrets
    #[serde(default)]
    pub refs: Vec<SecretRef>,
}

/// Secret reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRef {
    /// Secret name in Kubernetes
    pub name: String,
    /// Optional key in the secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Environment variable name
    pub env_var: String,
}

/// Build configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Docker context path (relative to app root)
    #[serde(default = "default_context")]
    pub context: String,
    /// Dockerfile path (relative to context)
    #[serde(default = "default_dockerfile")]
    pub dockerfile: String,
    /// Build arguments
    #[serde(default)]
    pub args: HashMap<String, String>,
    /// Target stage for multi-stage builds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Image registry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    /// Image repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Image tag (defaults to app version)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

fn default_context() -> String { ".".to_string() }
fn default_dockerfile() -> String { "Dockerfile".to_string() }

/// Deploy configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeployConfig {
    /// Namespace (defaults to app name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Helm chart configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helm: Option<HelmConfig>,
    /// Deployment strategy
    #[serde(default)]
    pub strategy: DeploymentStrategy,
    /// Service configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<ServiceConfig>,
    /// Ingress configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingress: Option<IngressConfig>,
}

/// Helm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmConfig {
    /// Use external chart instead of generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart: Option<String>,
    /// Chart repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Chart version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Values file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values_file: Option<String>,
    /// Inline values
    #[serde(default)]
    pub values: HashMap<String, serde_json::Value>,
}

/// Deployment strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentStrategy {
    /// Rolling update
    RollingUpdate {
        #[serde(default = "default_max_unavailable")]
        max_unavailable: String,
        #[serde(default = "default_max_surge")]
        max_surge: String,
    },
    /// Recreate
    Recreate,
    /// Blue-green
    BlueGreen,
    /// Canary
    Canary {
        #[serde(default = "default_canary_percentage")]
        percentage: u32,
    },
}

fn default_max_unavailable() -> String { "25%".to_string() }
fn default_max_surge() -> String { "25%".to_string() }
fn default_canary_percentage() -> u32 { 10 }

impl Default for DeploymentStrategy {
    fn default() -> Self {
        DeploymentStrategy::RollingUpdate {
            max_unavailable: default_max_unavailable(),
            max_surge: default_max_surge(),
        }
    }
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service type
    #[serde(default = "default_service_type")]
    pub service_type: ServiceType,
    /// Session affinity
    #[serde(default)]
    pub session_affinity: bool,
    /// Load balancer IP (for LoadBalancer type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_balancer_ip: Option<String>,
}

/// Service type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
    ExternalName,
}

fn default_service_type() -> ServiceType {
    ServiceType::ClusterIP
}

/// Ingress configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressConfig {
    /// Ingress class
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    /// Host rules
    pub hosts: Vec<IngressHost>,
    /// TLS configuration
    #[serde(default)]
    pub tls: Vec<IngressTls>,
    /// Annotations
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

/// Ingress host configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressHost {
    /// Hostname
    pub host: String,
    /// Paths
    #[serde(default)]
    pub paths: Vec<IngressPath>,
}

/// Ingress path configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressPath {
    /// Path
    pub path: String,
    /// Path type
    #[serde(default = "default_path_type")]
    pub path_type: String,
    /// Backend port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

fn default_path_type() -> String {
    "Prefix".to_string()
}

/// Ingress TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressTls {
    /// Hosts covered by this TLS certificate
    pub hosts: Vec<String>,
    /// Secret name containing the certificate
    pub secret_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manifest_deserialization() {
        let toml = r#"
        [app]
        name = "my-service"
        version = "1.0.0"
        description = "My awesome service"
        app_type = "service"
        
        [dependencies]
        postgresql = "^13.0"
        redis = { version = "^7.0", optional = true }
        
        [[config.ports]]
        name = "http"
        port = 8080
        
        [config.resources]
        cpu_request = "100m"
        memory_request = "128Mi"
        "#;
        
        let manifest: AppManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.app.name, "my-service");
        assert_eq!(manifest.app.version, "1.0.0");
        assert_eq!(manifest.dependencies.len(), 2);
    }
}

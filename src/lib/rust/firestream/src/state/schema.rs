//! State schema definitions
//!
//! This module defines the JSON schema for Firestream state files

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Root state file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirestreamState {
    /// State file version for migrations
    pub version: String,
    
    /// Unique serial number, incremented on each change
    pub serial: u64,
    
    /// Timestamp of last modification
    pub last_modified: DateTime<Utc>,
    
    /// User who made the last change
    pub last_modified_by: String,
    
    /// Infrastructure resources (managed by Pulumi)
    pub infrastructure: InfrastructureState,
    
    /// Container images that have been built
    pub builds: HashMap<String, BuildState>,
    
    /// Deployed services (Helm releases)
    pub deployments: HashMap<String, DeploymentState>,
    
    /// Cluster configuration
    pub cluster: ClusterState,
    
    /// State metadata
    pub metadata: StateMetadata,
}

impl FirestreamState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            serial: 0,
            last_modified: Utc::now(),
            last_modified_by: whoami::username(),
            infrastructure: InfrastructureState::default(),
            builds: HashMap::new(),
            deployments: HashMap::new(),
            cluster: ClusterState::default(),
            metadata: StateMetadata::default(),
        }
    }
    
    /// Increment serial number
    pub fn increment_serial(&mut self) {
        self.serial += 1;
        self.last_modified = Utc::now();
        self.last_modified_by = whoami::username();
    }
}

/// Infrastructure state (Pulumi-managed resources)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InfrastructureState {
    /// Pulumi stack name
    pub stack_name: Option<String>,
    
    /// Last Pulumi state refresh
    pub last_refresh: Option<DateTime<Utc>>,
    
    /// Pulumi outputs
    pub outputs: HashMap<String, serde_json::Value>,
    
    /// Resource URNs managed by Pulumi
    pub resource_urns: Vec<String>,
    
    /// Cloud provider
    pub provider: Option<CloudProvider>,
    
    /// Region/location
    pub region: Option<String>,
}

/// Supported cloud providers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    AWS,
    GCP,
    Azure,
    Local,
}

/// Build state for container images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildState {
    /// Image name without tag
    pub image_name: String,
    
    /// Image tag
    pub tag: String,
    
    /// Full image reference (registry/name:tag)
    pub image_ref: String,
    
    /// Build timestamp
    pub built_at: DateTime<Utc>,
    
    /// Git commit hash
    pub git_commit: Option<String>,
    
    /// Build platform (linux/amd64, linux/arm64, etc)
    pub platform: String,
    
    /// Image digest
    pub digest: Option<String>,
    
    /// Size in bytes
    pub size_bytes: Option<u64>,
    
    /// Build configuration used
    pub build_config: serde_json::Value,
}

/// Deployment state (Helm releases)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentState {
    /// Helm release name
    pub release_name: String,
    
    /// Helm chart reference
    pub chart: String,
    
    /// Chart version
    pub chart_version: String,
    
    /// Namespace
    pub namespace: String,
    
    /// Deployment status
    pub status: DeploymentStatus,
    
    /// Values used for deployment
    pub values: serde_json::Value,
    
    /// Deployment timestamp
    pub deployed_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Revision number
    pub revision: u32,
    
    /// Resources created
    pub resources: Vec<KubernetesResource>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Deployed,
    Failed,
    Superseded,
    Uninstalling,
    Uninstalled,
}

/// Kubernetes resource reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesResource {
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
}

/// Cluster state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClusterState {
    /// Cluster name
    pub name: Option<String>,
    
    /// Cluster type (k3d, eks, gke, aks, etc)
    pub cluster_type: Option<String>,
    
    /// Kubernetes version
    pub kubernetes_version: Option<String>,
    
    /// Cluster endpoint
    pub endpoint: Option<String>,
    
    /// Whether cluster is managed by Firestream
    pub managed: bool,
    
    /// K3D-specific configuration
    pub k3d_config: Option<K3dClusterConfig>,
}

/// K3D-specific cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K3dClusterConfig {
    /// Cluster name
    pub name: String,
    
    /// API server port
    pub api_port: u16,
    
    /// Load balancer HTTP port
    pub http_port: u16,
    
    /// Load balancer HTTPS port  
    pub https_port: u16,
    
    /// Number of server nodes
    pub servers: u32,
    
    /// Number of agent nodes
    pub agents: u32,
    
    /// K3s image version
    pub k3s_version: String,
    
    /// Registry configuration
    pub registry: K3dRegistryConfig,
    
    /// TLS configuration
    pub tls: K3dTlsConfig,
    
    /// Network configuration
    pub network: K3dNetworkConfig,
    
    /// Development mode settings
    pub dev_mode: Option<K3dDevModeConfig>,
}

impl Default for K3dClusterConfig {
    fn default() -> Self {
        Self {
            name: "firestream".to_string(),
            api_port: 6550,
            http_port: 80,
            https_port: 443,
            servers: 1,
            agents: 1,
            k3s_version: "v1.31.2-k3s1".to_string(),
            registry: K3dRegistryConfig::default(),
            tls: K3dTlsConfig::default(),
            network: K3dNetworkConfig::default(),
            dev_mode: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K3dRegistryConfig {
    pub enabled: bool,
    pub name: String,
    pub port: u16,
}

impl Default for K3dRegistryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            name: "registry.localhost".to_string(),
            port: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K3dTlsConfig {
    pub enabled: bool,
    pub secret_name: String,
    pub certificate_config: CertificateConfig,
}

impl Default for K3dTlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            secret_name: "firestream-tls".to_string(),
            certificate_config: CertificateConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateConfig {
    pub country: String,
    pub state: String,
    pub locality: String,
    pub organization: String,
    pub organizational_unit: String,
    pub common_name: String,
    pub email: String,
}

impl Default for CertificateConfig {
    fn default() -> Self {
        Self {
            country: "US".to_string(),
            state: "New Mexico".to_string(),
            locality: "Roswell".to_string(),
            organization: "ACME Co, LLC.".to_string(),
            organizational_unit: "ACME Department".to_string(),
            common_name: "www.domain.com".to_string(),
            email: "email@domain".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K3dNetworkConfig {
    pub configure_routes: bool,
    pub configure_dns: bool,
    pub patch_etc_hosts: bool,
    pub pod_cidr: String,
    pub service_cidr: String,
}

impl Default for K3dNetworkConfig {
    fn default() -> Self {
        Self {
            configure_routes: true,
            configure_dns: true,
            patch_etc_hosts: true,
            pod_cidr: "10.42.0.0/16".to_string(),
            service_cidr: "10.43.0.0/16".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K3dDevModeConfig {
    pub port_forward_all: bool,
    pub port_offset: u16,
}

/// State metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    /// Unique state ID
    pub state_id: String,
    
    /// Project name
    pub project_name: String,
    
    /// Environment (dev, staging, prod, etc)
    pub environment: String,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Tags for organization
    pub tags: HashMap<String, String>,
}

impl Default for StateMetadata {
    fn default() -> Self {
        Self {
            state_id: Uuid::new_v4().to_string(),
            project_name: "firestream".to_string(),
            environment: "development".to_string(),
            created_at: Utc::now(),
            tags: HashMap::new(),
        }
    }
}

/// Planned change representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedChange {
    /// Change ID for tracking
    pub id: String,
    
    /// Resource type (infrastructure, build, deployment)
    pub resource_type: ResourceType,
    
    /// Resource identifier
    pub resource_id: String,
    
    /// Type of change
    pub change_type: ChangeType,
    
    /// Human-readable description
    pub description: String,
    
    /// Old value (for updates and deletes)
    pub old_value: Option<serde_json::Value>,
    
    /// New value (for creates and updates)
    pub new_value: Option<serde_json::Value>,
    
    /// Whether this change requires confirmation
    pub requires_confirmation: bool,
    
    /// Estimated impact
    pub impact: ChangeImpact,
}

/// Resource types that can be changed
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Infrastructure,
    Build,
    Deployment,
    Cluster,
}

/// Types of changes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Create,
    Update,
    Delete,
    Replace,
    NoOp,
}

/// Impact level of changes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeImpact {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Plan ID
    pub id: String,
    
    /// Timestamp when plan was created
    pub created_at: DateTime<Utc>,
    
    /// User who created the plan
    pub created_by: String,
    
    /// List of planned changes
    pub changes: Vec<PlannedChange>,
    
    /// Current state serial number
    pub from_serial: u64,
    
    /// Target state serial number (after apply)
    pub to_serial: u64,
    
    /// Plan status
    pub status: PlanStatus,
    
    /// Execution options
    pub options: PlanOptions,
}

/// Plan status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlanStatus {
    Pending,
    Approved,
    Applying,
    Applied,
    Failed,
    Cancelled,
}

/// Plan execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOptions {
    /// Auto-approve without confirmation
    pub auto_approve: bool,
    
    /// Dry run only
    pub dry_run: bool,
    
    /// Target specific resources
    pub targets: Vec<String>,
    
    /// Skip certain resources
    pub skip: Vec<String>,
    
    /// Parallelism level
    pub parallelism: usize,
}

impl Default for PlanOptions {
    fn default() -> Self {
        Self {
            auto_approve: false,
            dry_run: false,
            targets: Vec::new(),
            skip: Vec::new(),
            parallelism: 10,
        }
    }
}

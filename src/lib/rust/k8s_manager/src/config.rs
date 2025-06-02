//! Configuration types for Kubernetes cluster management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// K3D registry configuration
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

/// K3D TLS configuration
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

/// TLS certificate configuration
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

/// K3D network configuration
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

/// K3D development mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K3dDevModeConfig {
    pub port_forward_all: bool,
    pub port_offset: u16,
}

/// Basic K3D configuration for simple setup
pub struct K3dConfig {
    pub cluster_name: String,
    pub api_port: u16,
    pub lb_port: u16,
    pub agents: u32,
    pub servers: u32,
    pub registry_name: String,
    pub registry_port: u16,
}

impl Default for K3dConfig {
    fn default() -> Self {
        Self {
            cluster_name: "firestream".to_string(),
            api_port: 6443,
            lb_port: 8080,
            agents: 1,
            servers: 1,
            registry_name: "registry.localhost".to_string(),
            registry_port: 5000,
        }
    }
}

/// Generic cluster information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub name: String,
    pub provider: ClusterProvider,
    pub status: ClusterStatus,
    pub endpoint: Option<String>,
    pub kubernetes_version: Option<String>,
    pub node_count: u32,
    pub metadata: HashMap<String, String>,
}

/// Supported cluster providers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClusterProvider {
    K3d,
    Gke,     // Google Kubernetes Engine
    Eks,     // Amazon Elastic Kubernetes Service
    Aks,     // Azure Kubernetes Service
    Custom,  // Custom/other providers
}

/// Cluster status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClusterStatus {
    Creating,
    Running,
    Updating,
    Deleting,
    Error,
    Unknown,
}

/// Port forwarding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardConfig {
    pub namespace: String,
    pub service_name: String,
    pub local_port: u16,
    pub remote_port: u16,
}

/// Logs query configuration
#[derive(Debug, Clone)]
pub struct LogsConfig {
    pub namespace: Option<String>,
    pub resource_type: ResourceType,
    pub resource_name: String,
    pub container: Option<String>,
    pub follow: bool,
    pub previous: bool,
    pub all_containers: bool,
}

/// Kubernetes resource type for logs
#[derive(Debug, Clone)]
pub enum ResourceType {
    Pod,
    Deployment,
    Service,
    StatefulSet,
    DaemonSet,
    Job,
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Pod => "pod",
            ResourceType::Deployment => "deployment",
            ResourceType::Service => "service",
            ResourceType::StatefulSet => "statefulset",
            ResourceType::DaemonSet => "daemonset",
            ResourceType::Job => "job",
        }
    }
}

/// Diagnostics configuration
#[derive(Debug, Clone, Default)]
pub struct DiagnosticsConfig {
    pub namespace: Option<String>,
    pub include_nodes: bool,
    pub include_pods: bool,
    pub include_services: bool,
    pub include_events: bool,
    pub include_all: bool,
}

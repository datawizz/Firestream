//! Configuration schema definitions
//!
//! This module defines all the configuration structures used by Firestream.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Global configuration for Firestream
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalConfig {
    pub version: String,
    pub cluster: ClusterConfig,
    pub defaults: DefaultSettings,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            cluster: ClusterConfig::default(),
            defaults: DefaultSettings::default(),
        }
    }
}

/// Cluster configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClusterConfig {
    pub name: String,
    pub kubeconfig_path: Option<PathBuf>,
    pub namespace: String,
    pub container_runtime: ContainerRuntime,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            kubeconfig_path: None,
            namespace: "firestream".to_string(),
            container_runtime: ContainerRuntime::Docker,
        }
    }
}

/// Container runtime options
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ContainerRuntime {
    Docker,
    Podman,
}

/// Default settings
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefaultSettings {
    pub log_level: LogLevel,
    pub timeout_seconds: u32,
    pub resource_limits: ResourceLimits,
}

impl Default for DefaultSettings {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            timeout_seconds: 300,
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Log level configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Resource limits configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceLimits {
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub disk_gb: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_cores: 2.0,
            memory_mb: 4096,
            disk_gb: 100,
        }
    }
}

/// Service configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceConfig {
    pub api_version: String,
    pub metadata: ServiceMetadata,
    pub spec: ServiceSpec,
}

/// Service metadata
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: ServiceCategory,
}

/// Service category
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ServiceCategory {
    Database,
    Streaming,
    Analytics,
    Monitoring,
    Storage,
    Other,
}

/// Service specification
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceSpec {
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub resources: ResourceRequirements,
    pub ports: Vec<PortMapping>,
    pub environment: HashMap<String, String>,
    pub volumes: Vec<VolumeMount>,
    pub health_check: Option<HealthCheck>,
}

/// Resource requirements for a service
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceRequirements {
    pub requests: ResourceSpec,
    pub limits: ResourceSpec,
}

/// Resource specification
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceSpec {
    pub cpu: String,
    pub memory: String,
}

/// Port mapping configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortMapping {
    pub name: String,
    pub container_port: u16,
    pub host_port: Option<u16>,
    pub protocol: Protocol,
}

/// Network protocol
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum Protocol {
    TCP,
    UDP,
}

/// Volume mount configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VolumeMount {
    pub name: String,
    pub mount_path: String,
    pub host_path: Option<String>,
    pub read_only: bool,
}

/// Health check configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HealthCheck {
    pub check_type: HealthCheckType,
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
    pub timeout_seconds: u32,
    pub success_threshold: u32,
    pub failure_threshold: u32,
}

/// Health check type
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum HealthCheckType {
    Http { path: String, port: u16 },
    Tcp { port: u16 },
    Exec { command: Vec<String> },
}

/// Service state information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceState {
    pub name: String,
    pub status: ServiceStatus,
    pub installed_version: Option<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub resource_usage: Option<ResourceUsage>,
}

/// Service status
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    NotInstalled,
    Installing,
    Running,
    Stopped,
    Error(String),
}

/// Resource usage information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub disk_gb: u32,
}

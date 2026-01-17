use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub name: String,
    pub provider: ClusterProvider,
    pub status: HealthStatus,
    pub nodes: NodeSummary,
    pub resources: ClusterResourceUtilization,
    pub services: HashMap<String, ServiceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClusterProvider {
    K3d,
    Gcp,
    Aws,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummary {
    pub total: u32,
    pub ready: u32,
    pub gpu: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResourceUtilization {
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: f64,
    #[serde(rename = "memoryUsage")]
    pub memory_usage: f64,
    #[serde(rename = "gpuUsage")]
    pub gpu_usage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub status: HealthStatus,
    pub replicas: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareTunnel {
    pub id: String,
    pub name: String,
    pub status: TunnelStatus,
    pub hostname: String,
    pub service: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Active,
    Inactive,
}

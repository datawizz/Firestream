use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub provider: NodeProvider,
    #[serde(rename = "instanceType")]
    pub instance_type: String,
    pub status: NodeStatus,
    pub resources: NodeResources,
    pub labels: HashMap<String, String>,
    pub spot: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeProvider {
    Gcp,
    Aws,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Ready,
    NotReady,
    Provisioning,
    Terminating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    pub cpu: u32,
    pub memory: String,
    pub gpu: Option<GpuInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub count: u32,
    #[serde(rename = "type")]
    pub gpu_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetail {
    #[serde(flatten)]
    pub node: Node,
    pub pods: Vec<super::deployment::Pod>,
    pub utilization: ResourceUtilization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu: f64,
    pub memory: f64,
    pub gpu: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    pub devices: Vec<GpuDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    pub index: u32,
    pub name: String,
    pub memory: GpuMemory,
    pub utilization: f64,
    pub temperature: f64,
    pub processes: Vec<GpuProcess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMemory {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuProcess {
    pub pid: u32,
    pub name: String,
    #[serde(rename = "memoryUsed")]
    pub memory_used: u64,
}

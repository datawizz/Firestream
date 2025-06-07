use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    #[serde(rename = "templateId")]
    pub template_id: String,
    pub namespace: String,
    pub status: DeploymentStatus,
    pub replicas: ReplicaStatus,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Running,
    Pending,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaStatus {
    pub desired: u32,
    pub ready: u32,
    pub available: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentDetail {
    #[serde(flatten)]
    pub deployment: Deployment,
    pub pods: Vec<Pod>,
    pub services: Vec<Service>,
    pub ingress: Vec<IngressRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub name: String,
    pub status: PodStatus,
    pub node: String,
    pub containers: Vec<Container>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PodStatus {
    Running,
    Pending,
    Failed,
    Succeeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    #[serde(rename = "type")]
    pub service_type: ServiceType,
    pub ports: Vec<ServicePort>,
    pub selector: HashMap<String, String>,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: Option<String>,
    pub port: u16,
    #[serde(rename = "targetPort")]
    pub target_port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRule {
    pub name: String,
    pub namespace: String,
    pub host: String,
    pub paths: Vec<IngressPath>,
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressPath {
    pub path: String,
    pub service: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    #[serde(rename = "secretName")]
    pub secret_name: Option<String>,
}

use std::collections::HashMap;

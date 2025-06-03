use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretInfo {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub secret_type: SecretType,
    pub keys: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecretType {
    Opaque,
    #[serde(rename = "kubernetes.io/tls")]
    TLS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateResult {
    #[serde(rename = "secretName")]
    pub secret_name: String,
    pub rotated: bool,
    #[serde(rename = "newVersion")]
    pub new_version: String,
    #[serde(rename = "oldVersion")]
    pub old_version: String,
    pub message: Option<String>,
}

// Models based on the OpenAPI specification

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod deployment;
pub mod template;
pub mod cluster;
pub mod node;
pub mod data;
pub mod build;
pub mod secret;

pub use deployment::*;
pub use template::*;
pub use cluster::*;
pub use node::*;
pub use data::*;
pub use build::*;
pub use secret::*;

// Common types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Deployment,
    Template,
    Node,
    Data,
    Build,
    Secret,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::Deployment => write!(f, "deployment"),
            ResourceType::Template => write!(f, "template"),
            ResourceType::Node => write!(f, "node"),
            ResourceType::Data => write!(f, "data"),
            ResourceType::Build => write!(f, "build"),
            ResourceType::Secret => write!(f, "secret"),
        }
    }
}

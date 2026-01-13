//! Container specification
//!
//! Defines the structure for container test configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{GossSpec, SecuritySpec};

/// Complete container test specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
    /// Container name
    pub name: String,
    /// Container version/tag
    pub version: String,
    /// Base image or Nix flake reference
    pub image: String,
    /// Goss test specification
    pub goss: Option<GossSpec>,
    /// Security scanning configuration
    pub security: Option<SecuritySpec>,
    /// Runtime configuration
    pub runtime: RuntimeConfig,
    /// Environment variables for testing
    pub env: HashMap<String, String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Runtime configuration for test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Runtime type (docker, kubernetes)
    #[serde(default = "default_runtime")]
    pub runtime: String,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Number of retry attempts
    #[serde(default)]
    pub retries: u32,
    /// Docker-specific configuration
    pub docker: Option<DockerConfig>,
    /// Kubernetes-specific configuration
    pub kubernetes: Option<KubernetesConfig>,
}

/// Docker runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    /// Docker daemon socket
    pub socket: Option<String>,
    /// Additional docker run flags
    pub run_flags: Vec<String>,
    /// Container network mode
    pub network: Option<String>,
}

/// Kubernetes runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Namespace to use
    pub namespace: String,
    /// Kubeconfig path
    pub kubeconfig: Option<String>,
    /// Pod template
    pub pod_template: Option<String>,
}

fn default_runtime() -> String {
    "docker".to_string()
}

fn default_timeout() -> u64 {
    300 // 5 minutes
}

impl ContainerSpec {
    /// Load a spec from a YAML file
    pub fn from_yaml_file(path: &str) -> Result<Self, super::Error> {
        let content = std::fs::read_to_string(path)?;
        let spec = serde_yaml::from_str(&content)?;
        Ok(spec)
    }

    /// Load a spec from a JSON file
    pub fn from_json_file(path: &str) -> Result<Self, super::Error> {
        let content = std::fs::read_to_string(path)?;
        let spec = serde_json::from_str(&content)?;
        Ok(spec)
    }

    /// Validate the spec
    pub fn validate(&self) -> Result<(), super::Error> {
        if self.name.is_empty() {
            return Err(super::Error::MissingField("name".to_string()));
        }
        if self.image.is_empty() {
            return Err(super::Error::MissingField("image".to_string()));
        }
        Ok(())
    }
}

//! Nix derivation operations
//!
//! Handles parsing and manipulation of Nix derivations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;

/// A Nix derivation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Derivation {
    /// Output paths
    pub outputs: HashMap<String, DerivationOutput>,
    /// Input derivations
    #[serde(rename = "inputDrvs")]
    pub input_drvs: HashMap<String, Vec<String>>,
    /// Input sources
    #[serde(rename = "inputSrcs")]
    pub input_srcs: Vec<String>,
    /// Platform/system
    pub platform: String,
    /// Builder executable
    pub builder: String,
    /// Builder arguments
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
}

/// Output of a derivation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationOutput {
    /// Output path
    pub path: Option<String>,
    /// Hash algorithm (optional)
    #[serde(rename = "hashAlgo")]
    pub hash_algo: Option<String>,
    /// Hash (optional)
    pub hash: Option<String>,
}

impl Derivation {
    /// Parse a derivation using `nix show-derivation`
    pub async fn parse(drv_path: &str) -> Result<Self, super::Error> {
        let output = Command::new("nix")
            .args(["show-derivation", drv_path])
            .output()
            .await
            .map_err(|e| super::Error::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(super::Error::CommandFailed(format!(
                "nix show-derivation failed: {}",
                stderr
            )));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);

        // The output is a map with the derivation path as key
        let drvs: HashMap<String, Derivation> = serde_json::from_str(&json_str).map_err(|e| {
            super::Error::ParseError(format!("Failed to parse derivation JSON: {}", e))
        })?;

        // Get the first (and usually only) derivation
        drvs.into_iter()
            .next()
            .map(|(_, drv)| drv)
            .ok_or_else(|| super::Error::ParseError("No derivation found in output".to_string()))
    }

    /// Parse a derivation from a .drv file (legacy format)
    pub fn from_file(path: &str) -> Result<Self, super::Error> {
        // For now, we'll use the parse method which is async
        // In a sync context, this would need to use blocking I/O
        Err(super::Error::ParseError(
            "Use Derivation::parse() for async parsing".to_string(),
        ))
    }

    /// Get the output path for a specific output name
    pub fn output_path(&self, name: &str) -> Option<&str> {
        self.outputs
            .get(name)
            .and_then(|o| o.path.as_ref().map(|s| s.as_str()))
    }

    /// Get the default output path
    pub fn default_output_path(&self) -> Option<&str> {
        self.output_path("out")
    }

    /// Get all output names
    pub fn output_names(&self) -> Vec<&str> {
        self.outputs.keys().map(|s| s.as_str()).collect()
    }

    /// Get the system/platform this derivation is built for
    pub fn system(&self) -> &str {
        &self.platform
    }

    /// Get the builder path
    pub fn builder_path(&self) -> &str {
        &self.builder
    }

    /// Get environment variable value
    pub fn env_var(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(|s| s.as_str())
    }

    /// Get all input derivation paths
    pub fn input_derivations(&self) -> Vec<&str> {
        self.input_drvs.keys().map(|s| s.as_str()).collect()
    }

    /// Get all input source paths
    pub fn input_sources(&self) -> &[String] {
        &self.input_srcs
    }
}

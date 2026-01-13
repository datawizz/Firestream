//! Nix flake operations
//!
//! Handles interaction with Nix flakes to build containers and extract information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;

/// Information about a Nix flake from `nix flake metadata`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeInfo {
    /// Original flake reference
    pub original: FlakeReference,
    /// Resolved flake reference
    pub resolved: FlakeReference,
    /// Locked flake reference
    pub locked: FlakeReference,
    /// Flake description
    pub description: Option<String>,
    /// Last modified timestamp
    #[serde(rename = "lastModified")]
    pub last_modified: Option<u64>,
    /// Path to the flake (if local)
    pub path: Option<String>,
    /// Revision (git commit hash)
    #[serde(rename = "revCount")]
    pub rev_count: Option<u64>,
    /// Revision
    pub revision: Option<String>,
}

/// A flake reference (original, resolved, or locked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeReference {
    /// Type of reference (e.g., "path", "github", "indirect")
    #[serde(rename = "type")]
    pub ref_type: String,
    /// Additional attributes (depends on type)
    #[serde(flatten)]
    pub attrs: HashMap<String, serde_json::Value>,
}

impl FlakeInfo {
    /// Create a FlakeInfo from a flake reference
    pub async fn from_ref(flake_ref: &str) -> Result<Self, super::Error> {
        metadata(flake_ref).await
    }

    /// Build the flake and return the output store path
    pub async fn build(&self, output: Option<&str>) -> Result<String, super::Error> {
        // Construct the flake reference from the locked or resolved reference
        let flake_ref = self.to_flake_ref();
        build(&flake_ref, output.unwrap_or("")).await
    }

    /// Convert FlakeInfo back to a flake reference string
    fn to_flake_ref(&self) -> String {
        // Use path if available (for local flakes)
        if let Some(path) = &self.path {
            return path.clone();
        }

        // Otherwise, try to construct from locked reference
        match self.locked.ref_type.as_str() {
            "github" => {
                let owner = self.locked.attrs.get("owner").and_then(|v| v.as_str());
                let repo = self.locked.attrs.get("repo").and_then(|v| v.as_str());
                let rev = self.locked.attrs.get("rev").and_then(|v| v.as_str());

                if let (Some(owner), Some(repo)) = (owner, repo) {
                    if let Some(rev) = rev {
                        format!("github:{}/{}?rev={}", owner, repo, rev)
                    } else {
                        format!("github:{}/{}", owner, repo)
                    }
                } else {
                    // Fallback to original
                    format!("{:?}", self.original)
                }
            }
            _ => {
                // For other types, fallback to path or original
                format!("{:?}", self.original)
            }
        }
    }
}

/// Build a Nix flake and return the store path
pub async fn build(flake_ref: &str, output: &str) -> Result<String, super::Error> {
    let flake_attr = if output.is_empty() {
        flake_ref.to_string()
    } else {
        format!("{}#{}", flake_ref, output)
    };

    let output = Command::new("nix")
        .args(["build", "--no-link", "--print-out-paths", &flake_attr])
        .output()
        .await
        .map_err(|e| super::Error::Io(e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(super::Error::CommandFailed(format!(
            "nix build failed: {}",
            stderr
        )));
    }

    let store_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(store_path)
}

/// Get metadata about a Nix flake
pub async fn metadata(flake_ref: &str) -> Result<FlakeInfo, super::Error> {
    let output = Command::new("nix")
        .args(["flake", "metadata", "--json", flake_ref])
        .output()
        .await
        .map_err(|e| super::Error::Io(e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(super::Error::CommandFailed(format!(
            "nix flake metadata failed: {}",
            stderr
        )));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&json_str).map_err(|e| {
        super::Error::ParseError(format!("Failed to parse flake metadata JSON: {}", e))
    })
}

/// List all outputs available in a flake
pub async fn list_outputs(flake_ref: &str) -> Result<Vec<String>, super::Error> {
    let output = Command::new("nix")
        .args(["flake", "show", "--json", flake_ref])
        .output()
        .await
        .map_err(|e| super::Error::Io(e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(super::Error::CommandFailed(format!(
            "nix flake show failed: {}",
            stderr
        )));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let show_data: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
        super::Error::ParseError(format!("Failed to parse flake show JSON: {}", e))
    })?;

    // Extract output names from the flake show structure
    // The structure is typically: { "packages": { "system": { "name": {...} } } }
    let mut outputs = Vec::new();

    if let Some(packages) = show_data.get("packages").and_then(|v| v.as_object()) {
        for (_system, pkgs) in packages {
            if let Some(pkg_obj) = pkgs.as_object() {
                for (name, _) in pkg_obj {
                    outputs.push(name.clone());
                }
            }
        }
    }

    Ok(outputs)
}

/// Check if a flake output exists
pub async fn has_output(flake_ref: &str, output: &str) -> Result<bool, super::Error> {
    let outputs = list_outputs(flake_ref).await?;
    Ok(outputs.iter().any(|o| o == output))
}

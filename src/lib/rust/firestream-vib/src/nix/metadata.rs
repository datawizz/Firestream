//! Nix metadata extraction
//!
//! Extracts and parses metadata from Nix derivations and store paths.

use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// Metadata about a Nix-built container from flake's testMetadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixMetadata {
    /// Container name
    pub name: String,
    /// Container version
    pub version: String,
    /// Goss test configuration
    pub goss: GossMetadata,
    /// Security scanning configuration
    pub security: SecurityMetadata,
    /// Exposed ports
    #[serde(default)]
    pub exposed_ports: Vec<u16>,
    /// Container user
    #[serde(default)]
    pub user: String,
    /// Working directory
    #[serde(default)]
    pub workdir: String,
}

/// Goss test metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossMetadata {
    /// Binaries to check
    #[serde(default)]
    pub binaries: Vec<String>,
    /// Directories to check
    #[serde(default)]
    pub directories: Vec<DirectoryCheck>,
    /// Version check configuration
    pub version: VersionCheck,
    /// Check for broken symlinks
    #[serde(default)]
    pub check_symlinks: bool,
    /// Check linked libraries in Nix store
    #[serde(default)]
    pub check_linked_libraries: Option<LinkedLibrariesCheck>,
    /// Check CA certificates
    #[serde(default)]
    pub check_ca_certs: bool,
    /// Check SPDX metadata
    #[serde(default)]
    pub check_spdx: bool,
}

/// Directory check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryCheck {
    /// Directory path
    pub path: String,
    /// Expected mode (optional)
    #[serde(default)]
    pub mode: Option<String>,
}

/// Version check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionCheck {
    /// Command to run
    pub command: String,
    /// Arguments to pass
    #[serde(default)]
    pub args: Vec<String>,
    /// Expected version string
    pub expected: String,
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    30000
}

/// Linked libraries check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedLibrariesCheck {
    /// Whether to check linked libraries
    pub enabled: bool,
    /// Root directory to check
    pub root_dir: String,
    /// Patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// Security scanning metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetadata {
    /// Trivy configuration
    pub trivy: TrivyMetadata,
    /// Grype configuration
    pub grype: GrypeMetadata,
}

/// Trivy scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrivyMetadata {
    /// Severity threshold
    pub severity_threshold: String,
    /// Vulnerability types to scan
    #[serde(default)]
    pub vuln_types: Vec<String>,
    /// Ignore unfixed vulnerabilities
    #[serde(default)]
    pub ignore_unfixed: bool,
}

/// Grype scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrypeMetadata {
    /// Fail on severity level
    pub fail_on_severity: String,
    /// Package types to scan
    #[serde(default)]
    pub package_types: Vec<String>,
}

/// Store path metadata for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorePathMetadata {
    /// Store path
    pub store_path: String,
    /// NAR hash
    pub nar_hash: String,
    /// NAR size in bytes
    pub nar_size: u64,
    /// Runtime dependencies (closure)
    pub references: Vec<String>,
    /// Derivation path
    pub deriver: Option<String>,
}

impl NixMetadata {
    /// Extract testMetadata from a Nix flake
    pub async fn from_flake(flake_path: &str) -> Result<Self, super::Error> {
        let output = Command::new("nix")
            .args([
                "eval",
                "--json",
                &format!("{}#testMetadata", flake_path),
            ])
            .output()
            .await
            .map_err(|e| super::Error::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(super::Error::CommandFailed(format!(
                "nix eval failed: {}",
                stderr
            )));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&json_str).map_err(|e| {
            super::Error::ParseError(format!("Failed to parse testMetadata JSON: {}", e))
        })
    }

    /// Extract testMetadata from a derivation's output
    pub async fn from_derivation(drv_path: &str) -> Result<Self, super::Error> {
        // First, build the derivation to get the store path
        let build_output = Command::new("nix-store")
            .args(["--realise", drv_path])
            .output()
            .await
            .map_err(|e| super::Error::Io(e))?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            return Err(super::Error::CommandFailed(format!(
                "nix-store --realise failed: {}",
                stderr
            )));
        }

        let store_path = String::from_utf8_lossy(&build_output.stdout).trim().to_string();

        // Read the testMetadata file from the store path
        let metadata_path = format!("{}/test-metadata.json", store_path);
        let content = tokio::fs::read_to_string(&metadata_path)
            .await
            .map_err(|e| {
                super::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("testMetadata file not found at {}: {}", metadata_path, e),
                ))
            })?;

        serde_json::from_str(&content).map_err(|e| {
            super::Error::ParseError(format!("Failed to parse testMetadata JSON: {}", e))
        })
    }

    /// Get the cache key for this metadata (derived from version and name)
    pub fn cache_key(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

impl StorePathMetadata {
    /// Extract metadata from a store path
    pub async fn from_store_path(path: &str) -> Result<Self, super::Error> {
        // Get path info including NAR hash and size
        let info_output = Command::new("nix")
            .args(["path-info", "--json", path])
            .output()
            .await
            .map_err(|e| super::Error::Io(e))?;

        if !info_output.status.success() {
            let stderr = String::from_utf8_lossy(&info_output.stderr);
            return Err(super::Error::CommandFailed(format!(
                "nix path-info failed: {}",
                stderr
            )));
        }

        let json_str = String::from_utf8_lossy(&info_output.stdout);
        let info_array: Vec<serde_json::Value> = serde_json::from_str(&json_str).map_err(|e| {
            super::Error::ParseError(format!("Failed to parse path-info JSON: {}", e))
        })?;

        let info = info_array
            .first()
            .ok_or_else(|| super::Error::ParseError("Empty path-info result".to_string()))?;

        Ok(StorePathMetadata {
            store_path: info["path"]
                .as_str()
                .ok_or_else(|| super::Error::ParseError("Missing path field".to_string()))?
                .to_string(),
            nar_hash: info["narHash"]
                .as_str()
                .ok_or_else(|| super::Error::ParseError("Missing narHash field".to_string()))?
                .to_string(),
            nar_size: info["narSize"]
                .as_u64()
                .ok_or_else(|| super::Error::ParseError("Missing narSize field".to_string()))?,
            references: info["references"]
                .as_array()
                .ok_or_else(|| super::Error::ParseError("Missing references field".to_string()))?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            deriver: info["deriver"].as_str().map(String::from),
        })
    }

    /// Get the hash for cache key generation
    pub fn cache_key(&self) -> String {
        self.nar_hash.clone()
    }

    /// Get total closure size
    pub async fn closure_size(&self) -> Result<u64, super::Error> {
        // Get recursive closure info
        let closure_output = Command::new("nix")
            .args(["path-info", "--json", "--recursive", &self.store_path])
            .output()
            .await
            .map_err(|e| super::Error::Io(e))?;

        if !closure_output.status.success() {
            let stderr = String::from_utf8_lossy(&closure_output.stderr);
            return Err(super::Error::CommandFailed(format!(
                "nix path-info --recursive failed: {}",
                stderr
            )));
        }

        let json_str = String::from_utf8_lossy(&closure_output.stdout);
        let closure_array: Vec<serde_json::Value> =
            serde_json::from_str(&json_str).map_err(|e| {
                super::Error::ParseError(format!("Failed to parse closure JSON: {}", e))
            })?;

        let total_size: u64 = closure_array
            .iter()
            .filter_map(|item| item["narSize"].as_u64())
            .sum();

        Ok(total_size)
    }

    /// Get all runtime dependencies recursively
    pub async fn full_closure(&self) -> Result<Vec<String>, super::Error> {
        let closure_output = Command::new("nix")
            .args(["path-info", "--json", "--recursive", &self.store_path])
            .output()
            .await
            .map_err(|e| super::Error::Io(e))?;

        if !closure_output.status.success() {
            let stderr = String::from_utf8_lossy(&closure_output.stderr);
            return Err(super::Error::CommandFailed(format!(
                "nix path-info --recursive failed: {}",
                stderr
            )));
        }

        let json_str = String::from_utf8_lossy(&closure_output.stdout);
        let closure_array: Vec<serde_json::Value> =
            serde_json::from_str(&json_str).map_err(|e| {
                super::Error::ParseError(format!("Failed to parse closure JSON: {}", e))
            })?;

        Ok(closure_array
            .iter()
            .filter_map(|item| item["path"].as_str().map(String::from))
            .collect())
    }
}

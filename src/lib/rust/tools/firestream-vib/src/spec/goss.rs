//! Goss test specification
//!
//! Defines the structure for Goss-based structural tests.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Goss test specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossSpec {
    /// Path to goss.yaml template or file
    pub template: Option<String>,
    /// Inline goss tests
    pub tests: Option<GossTests>,
    /// Template variables
    pub vars: HashMap<String, serde_json::Value>,
}

/// Goss test definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossTests {
    /// File existence and content tests
    #[serde(default)]
    pub file: HashMap<String, FileTest>,
    /// Package installation tests
    #[serde(default)]
    pub package: HashMap<String, PackageTest>,
    /// Service status tests
    #[serde(default)]
    pub service: HashMap<String, ServiceTest>,
    /// Port listening tests
    #[serde(default)]
    pub port: HashMap<String, PortTest>,
    /// Process running tests
    #[serde(default)]
    pub process: HashMap<String, ProcessTest>,
    /// Command execution tests
    #[serde(default)]
    pub command: HashMap<String, CommandTest>,
    /// HTTP endpoint tests
    #[serde(default)]
    pub http: HashMap<String, HttpTest>,
}

/// File test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTest {
    /// File must exist
    pub exists: bool,
    /// File mode/permissions
    pub mode: Option<String>,
    /// File owner
    pub owner: Option<String>,
    /// File group
    pub group: Option<String>,
    /// File content patterns
    pub contains: Option<Vec<String>>,
}

/// Package test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTest {
    /// Package must be installed
    pub installed: bool,
    /// Expected version
    pub version: Option<String>,
}

/// Service test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTest {
    /// Service must be enabled
    pub enabled: Option<bool>,
    /// Service must be running
    pub running: Option<bool>,
}

/// Port test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortTest {
    /// Port must be listening
    pub listening: bool,
    /// IP address
    pub ip: Option<Vec<String>>,
}

/// Process test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTest {
    /// Process must be running
    pub running: bool,
}

/// Command test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandTest {
    /// Expected exit code
    pub exit_status: i32,
    /// Expected stdout patterns
    pub stdout: Option<Vec<String>>,
    /// Expected stderr patterns
    pub stderr: Option<Vec<String>>,
    /// Command timeout
    pub timeout: Option<u64>,
}

/// HTTP test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTest {
    /// Expected HTTP status code
    pub status: u16,
    /// Expected response headers
    pub headers: Option<HashMap<String, String>>,
    /// Expected response body patterns
    pub body: Option<Vec<String>>,
    /// Request timeout
    pub timeout: Option<u64>,
}

impl GossSpec {
    /// Create a new empty Goss spec
    pub fn new() -> Self {
        Self {
            template: None,
            tests: None,
            vars: HashMap::new(),
        }
    }

    /// Load from a goss.yaml file
    pub fn from_file(path: &str) -> Result<Self, super::Error> {
        let content = std::fs::read_to_string(path)?;
        let spec = serde_yaml::from_str(&content)?;
        Ok(spec)
    }
}

impl Default for GossSpec {
    fn default() -> Self {
        Self::new()
    }
}

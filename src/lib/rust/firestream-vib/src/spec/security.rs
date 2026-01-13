//! Security scanning specification
//!
//! Defines security scanning requirements and thresholds.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security scanning specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    /// Trivy scanner configuration
    pub trivy: Option<TrivyConfig>,
    /// Grype scanner configuration
    pub grype: Option<GrypeConfig>,
    /// Vulnerability severity thresholds
    pub thresholds: SeverityThresholds,
    /// CVE allowlist (vulnerabilities to ignore)
    pub allowlist: Vec<String>,
}

/// Trivy scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrivyConfig {
    /// Enable Trivy scanning
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Trivy binary path
    pub binary: Option<String>,
    /// Scan timeout in seconds
    #[serde(default = "default_scan_timeout")]
    pub timeout: u64,
    /// Additional Trivy arguments
    pub args: Vec<String>,
}

/// Grype scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrypeConfig {
    /// Enable Grype scanning
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Grype binary path
    pub binary: Option<String>,
    /// Scan timeout in seconds
    #[serde(default = "default_scan_timeout")]
    pub timeout: u64,
    /// Additional Grype arguments
    pub args: Vec<String>,
}

/// Vulnerability severity thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityThresholds {
    /// Maximum allowed CRITICAL vulnerabilities
    #[serde(default)]
    pub critical: u32,
    /// Maximum allowed HIGH vulnerabilities
    #[serde(default)]
    pub high: u32,
    /// Maximum allowed MEDIUM vulnerabilities
    #[serde(default)]
    pub medium: u32,
    /// Maximum allowed LOW vulnerabilities
    #[serde(default)]
    pub low: u32,
    /// Fail on any vulnerability above threshold
    #[serde(default = "default_true")]
    pub fail_on_threshold: bool,
}

fn default_true() -> bool {
    true
}

fn default_scan_timeout() -> u64 {
    600 // 10 minutes
}

impl SecuritySpec {
    /// Create a new security spec with default values
    pub fn new() -> Self {
        Self {
            trivy: Some(TrivyConfig {
                enabled: true,
                binary: None,
                timeout: default_scan_timeout(),
                args: vec![],
            }),
            grype: Some(GrypeConfig {
                enabled: true,
                binary: None,
                timeout: default_scan_timeout(),
                args: vec![],
            }),
            thresholds: SeverityThresholds::default(),
            allowlist: vec![],
        }
    }

    /// Check if security scanning is enabled
    pub fn is_enabled(&self) -> bool {
        self.trivy.as_ref().map(|t| t.enabled).unwrap_or(false)
            || self.grype.as_ref().map(|g| g.enabled).unwrap_or(false)
    }
}

impl Default for SecuritySpec {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SeverityThresholds {
    fn default() -> Self {
        Self {
            critical: 0,
            high: 0,
            medium: u32::MAX,
            low: u32::MAX,
            fail_on_threshold: true,
        }
    }
}

/// Vulnerability severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

/// Security scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResult {
    /// Scanner name
    pub scanner: String,
    /// Scan timestamp
    pub timestamp: String,
    /// Vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,
    /// Summary by severity
    pub summary: HashMap<String, u32>,
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// CVE ID
    pub id: String,
    /// Vulnerability severity
    pub severity: Severity,
    /// Affected package
    pub package: String,
    /// Installed version
    pub installed_version: String,
    /// Fixed version (if available)
    pub fixed_version: Option<String>,
    /// Description
    pub description: String,
}

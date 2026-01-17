//! Trivy security scanner runner
//!
//! Executes Trivy vulnerability scans against container images.

use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use serde::Deserialize;

use crate::spec::security::{SecurityResult, Vulnerability, Severity};

/// Trivy scanner runner
pub struct TrivyRunner {
    /// Path to trivy binary
    binary: String,
}

impl TrivyRunner {
    /// Create a new Trivy runner
    pub fn new() -> Self {
        Self {
            binary: "trivy".to_string(),
        }
    }

    /// Create a Trivy runner with a custom binary path
    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    /// Scan a container image
    pub async fn scan_image(&self, image: &str) -> Result<SecurityResult, super::Error> {
        let output = Command::new(&self.binary)
            .args([
                "image",
                "--format",
                "json",
                "--quiet",
                image,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(super::Error::SecurityScanFailed(error.to_string()));
        }

        let result_json = String::from_utf8_lossy(&output.stdout);
        self.parse_results(&result_json)
    }

    /// Scan a filesystem path
    pub async fn scan_filesystem(&self, path: &str) -> Result<SecurityResult, super::Error> {
        let output = Command::new(&self.binary)
            .args([
                "filesystem",
                "--format",
                "json",
                "--quiet",
                path,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(super::Error::SecurityScanFailed(error.to_string()));
        }

        let result_json = String::from_utf8_lossy(&output.stdout);
        self.parse_results(&result_json)
    }

    /// Parse Trivy JSON output into SecurityResult
    fn parse_results(&self, json: &str) -> Result<SecurityResult, super::Error> {
        let trivy_output: TrivyOutput = serde_json::from_str(json)
            .map_err(|e| super::Error::SecurityScanFailed(
                format!("Failed to parse Trivy output: {}", e)
            ))?;

        let mut vulnerabilities = Vec::new();
        let mut summary: HashMap<String, u32> = HashMap::new();

        // Initialize summary counts
        summary.insert("Critical".to_string(), 0);
        summary.insert("High".to_string(), 0);
        summary.insert("Medium".to_string(), 0);
        summary.insert("Low".to_string(), 0);
        summary.insert("Unknown".to_string(), 0);

        // Process each result
        for result in trivy_output.results.unwrap_or_default() {
            for vuln in result.vulnerabilities.unwrap_or_default() {
                let severity = parse_trivy_severity(&vuln.severity);

                // Update summary counts
                let severity_key = match severity {
                    Severity::Critical => "Critical",
                    Severity::High => "High",
                    Severity::Medium => "Medium",
                    Severity::Low => "Low",
                    Severity::Unknown => "Unknown",
                };
                *summary.entry(severity_key.to_string()).or_insert(0) += 1;

                vulnerabilities.push(Vulnerability {
                    id: vuln.vulnerability_id,
                    severity,
                    package: vuln.pkg_name,
                    installed_version: vuln.installed_version,
                    fixed_version: vuln.fixed_version,
                    description: vuln.description.unwrap_or_else(||
                        vuln.title.unwrap_or_default()
                    ),
                });
            }
        }

        Ok(SecurityResult {
            scanner: "trivy".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            vulnerabilities,
            summary,
        })
    }

    /// Check if Trivy is installed
    pub async fn is_available(&self) -> bool {
        Command::new(&self.binary)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

impl Default for TrivyRunner {
    fn default() -> Self {
        Self::new()
    }
}

// Trivy JSON output structures
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivyOutput {
    results: Option<Vec<TrivyResult>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivyResult {
    vulnerabilities: Option<Vec<TrivyVulnerability>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivyVulnerability {
    #[serde(rename = "VulnerabilityID")]
    vulnerability_id: String,
    pkg_name: String,
    installed_version: String,
    fixed_version: Option<String>,
    severity: String,
    title: Option<String>,
    description: Option<String>,
}

/// Parse Trivy severity string to Severity enum
fn parse_trivy_severity(severity: &str) -> Severity {
    match severity.to_uppercase().as_str() {
        "CRITICAL" => Severity::Critical,
        "HIGH" => Severity::High,
        "MEDIUM" => Severity::Medium,
        "LOW" => Severity::Low,
        _ => Severity::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trivy_output() {
        let json_output = r#"{
            "SchemaVersion": 2,
            "Results": [
                {
                    "Target": "alpine:3.18",
                    "Class": "os-pkgs",
                    "Type": "alpine",
                    "Vulnerabilities": [
                        {
                            "VulnerabilityID": "CVE-2023-12345",
                            "PkgName": "openssl",
                            "InstalledVersion": "1.1.1",
                            "FixedVersion": "1.1.2",
                            "Severity": "HIGH",
                            "Title": "Test vulnerability",
                            "Description": "A test vulnerability in openssl"
                        },
                        {
                            "VulnerabilityID": "CVE-2023-67890",
                            "PkgName": "libcrypto",
                            "InstalledVersion": "1.0.0",
                            "Severity": "CRITICAL"
                        }
                    ]
                }
            ]
        }"#;

        let runner = TrivyRunner::new();
        let result = runner.parse_results(json_output).unwrap();

        assert_eq!(result.scanner, "trivy");
        assert_eq!(result.vulnerabilities.len(), 2);

        let first_vuln = &result.vulnerabilities[0];
        assert_eq!(first_vuln.id, "CVE-2023-12345");
        assert_eq!(first_vuln.package, "openssl");
        assert_eq!(first_vuln.installed_version, "1.1.1");
        assert_eq!(first_vuln.fixed_version, Some("1.1.2".to_string()));
        assert!(matches!(first_vuln.severity, Severity::High));

        let second_vuln = &result.vulnerabilities[1];
        assert_eq!(second_vuln.id, "CVE-2023-67890");
        assert!(matches!(second_vuln.severity, Severity::Critical));
        assert_eq!(second_vuln.fixed_version, None);

        assert_eq!(result.summary.get("High"), Some(&1));
        assert_eq!(result.summary.get("Critical"), Some(&1));
    }

    #[test]
    fn test_parse_trivy_severity() {
        assert!(matches!(parse_trivy_severity("CRITICAL"), Severity::Critical));
        assert!(matches!(parse_trivy_severity("critical"), Severity::Critical));
        assert!(matches!(parse_trivy_severity("HIGH"), Severity::High));
        assert!(matches!(parse_trivy_severity("high"), Severity::High));
        assert!(matches!(parse_trivy_severity("MEDIUM"), Severity::Medium));
        assert!(matches!(parse_trivy_severity("LOW"), Severity::Low));
        assert!(matches!(parse_trivy_severity("UNKNOWN"), Severity::Unknown));
        assert!(matches!(parse_trivy_severity("invalid"), Severity::Unknown));
    }

    #[test]
    fn test_parse_empty_results() {
        let json_output = r#"{
            "SchemaVersion": 2,
            "Results": []
        }"#;

        let runner = TrivyRunner::new();
        let result = runner.parse_results(json_output).unwrap();

        assert_eq!(result.vulnerabilities.len(), 0);
        assert_eq!(result.summary.get("Critical"), Some(&0));
    }
}

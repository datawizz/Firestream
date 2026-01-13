//! Grype security scanner runner
//!
//! Executes Grype vulnerability scans against container images.

use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use serde::Deserialize;

use crate::spec::security::{SecurityResult, Vulnerability, Severity};

/// Grype scanner runner
pub struct GrypeRunner {
    /// Path to grype binary
    binary: String,
}

impl GrypeRunner {
    /// Create a new Grype runner
    pub fn new() -> Self {
        Self {
            binary: "grype".to_string(),
        }
    }

    /// Create a Grype runner with a custom binary path
    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    /// Scan a container image
    pub async fn scan_image(&self, image: &str) -> Result<SecurityResult, super::Error> {
        let output = Command::new(&self.binary)
            .args([
                image,
                "-o",
                "json",
                "-q",
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

    /// Scan an SBOM file
    pub async fn scan_sbom(&self, sbom_path: &str) -> Result<SecurityResult, super::Error> {
        let sbom_input = format!("sbom:{}", sbom_path);

        let output = Command::new(&self.binary)
            .args([
                &sbom_input,
                "-o",
                "json",
                "-q",
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

    /// Parse Grype JSON output into SecurityResult
    fn parse_results(&self, json: &str) -> Result<SecurityResult, super::Error> {
        let grype_output: GrypeOutput = serde_json::from_str(json)
            .map_err(|e| super::Error::SecurityScanFailed(
                format!("Failed to parse Grype output: {}", e)
            ))?;

        let mut vulnerabilities = Vec::new();
        let mut summary: HashMap<String, u32> = HashMap::new();

        // Initialize summary counts
        summary.insert("Critical".to_string(), 0);
        summary.insert("High".to_string(), 0);
        summary.insert("Medium".to_string(), 0);
        summary.insert("Low".to_string(), 0);
        summary.insert("Unknown".to_string(), 0);

        // Process each match
        for match_result in grype_output.matches {
            let severity = parse_grype_severity(&match_result.vulnerability.severity);

            // Update summary counts
            let severity_key = match severity {
                Severity::Critical => "Critical",
                Severity::High => "High",
                Severity::Medium => "Medium",
                Severity::Low => "Low",
                Severity::Unknown => "Unknown",
            };
            *summary.entry(severity_key.to_string()).or_insert(0) += 1;

            // Extract fixed version if available
            let fixed_version = match &match_result.vulnerability.fix {
                Some(fix) => fix.versions.first().cloned(),
                None => None,
            };

            vulnerabilities.push(Vulnerability {
                id: match_result.vulnerability.id,
                severity,
                package: match_result.artifact.name,
                installed_version: match_result.artifact.version,
                fixed_version,
                description: match_result.vulnerability.description.unwrap_or_default(),
            });
        }

        Ok(SecurityResult {
            scanner: "grype".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            vulnerabilities,
            summary,
        })
    }

    /// Check if Grype is installed
    pub async fn is_available(&self) -> bool {
        Command::new(&self.binary)
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

impl Default for GrypeRunner {
    fn default() -> Self {
        Self::new()
    }
}

// Grype JSON output structures
#[derive(Debug, Deserialize)]
struct GrypeOutput {
    matches: Vec<GrypeMatch>,
}

#[derive(Debug, Deserialize)]
struct GrypeMatch {
    vulnerability: GrypeVulnerability,
    artifact: GrypeArtifact,
}

#[derive(Debug, Deserialize)]
struct GrypeVulnerability {
    id: String,
    severity: String,
    description: Option<String>,
    fix: Option<GrypeFix>,
}

#[derive(Debug, Deserialize)]
struct GrypeFix {
    versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GrypeArtifact {
    name: String,
    version: String,
}

/// Parse Grype severity string to Severity enum
fn parse_grype_severity(severity: &str) -> Severity {
    match severity.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        _ => Severity::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_grype_output() {
        let json_output = r#"{
            "matches": [
                {
                    "vulnerability": {
                        "id": "CVE-2023-12345",
                        "severity": "High",
                        "description": "A test vulnerability",
                        "fix": {
                            "versions": ["1.1.2"],
                            "state": "fixed"
                        }
                    },
                    "artifact": {
                        "name": "openssl",
                        "version": "1.1.1",
                        "type": "apk"
                    }
                },
                {
                    "vulnerability": {
                        "id": "CVE-2023-67890",
                        "severity": "Critical",
                        "description": "Critical vulnerability without fix"
                    },
                    "artifact": {
                        "name": "libcrypto",
                        "version": "1.0.0",
                        "type": "apk"
                    }
                }
            ]
        }"#;

        let runner = GrypeRunner::new();
        let result = runner.parse_results(json_output).unwrap();

        assert_eq!(result.scanner, "grype");
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
    fn test_parse_grype_severity() {
        assert!(matches!(parse_grype_severity("Critical"), Severity::Critical));
        assert!(matches!(parse_grype_severity("CRITICAL"), Severity::Critical));
        assert!(matches!(parse_grype_severity("High"), Severity::High));
        assert!(matches!(parse_grype_severity("HIGH"), Severity::High));
        assert!(matches!(parse_grype_severity("Medium"), Severity::Medium));
        assert!(matches!(parse_grype_severity("Low"), Severity::Low));
        assert!(matches!(parse_grype_severity("Unknown"), Severity::Unknown));
        assert!(matches!(parse_grype_severity("invalid"), Severity::Unknown));
    }

    #[test]
    fn test_parse_empty_matches() {
        let json_output = r#"{
            "matches": []
        }"#;

        let runner = GrypeRunner::new();
        let result = runner.parse_results(json_output).unwrap();

        assert_eq!(result.vulnerabilities.len(), 0);
        assert_eq!(result.summary.get("Critical"), Some(&0));
    }
}

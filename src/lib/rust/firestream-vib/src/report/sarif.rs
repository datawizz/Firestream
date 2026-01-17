//! SARIF report generation
//!
//! Generates security scan reports in SARIF format for GitHub Security.

use crate::spec::security::{SecurityResult, Severity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SARIF_SCHEMA: &str = "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";

/// SARIF report (SARIF 2.1.0 compliant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifReport {
    /// Schema URI
    #[serde(rename = "$schema")]
    pub schema: String,
    /// SARIF version
    pub version: String,
    /// Runs (scan results)
    pub runs: Vec<SarifRun>,
}

/// SARIF run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRun {
    /// Tool information
    pub tool: SarifTool,
    /// Results
    pub results: Vec<SarifResult>,
}

/// SARIF tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifTool {
    /// Driver (tool details)
    pub driver: SarifDriver,
}

/// SARIF driver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifDriver {
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Information URI
    #[serde(rename = "informationUri", skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<String>,
    /// Rules
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<SarifRule>,
}

/// SARIF rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Short description
    #[serde(rename = "shortDescription")]
    pub short_description: SarifMessage,
    /// Full description
    #[serde(rename = "fullDescription")]
    pub full_description: SarifMessage,
    /// Help URI
    #[serde(rename = "helpUri", skip_serializing_if = "Option::is_none")]
    pub help_uri: Option<String>,
    /// Default configuration
    #[serde(rename = "defaultConfiguration")]
    pub default_configuration: SarifConfiguration,
}

/// SARIF rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifConfiguration {
    /// Severity level
    pub level: String,
}

/// SARIF result (finding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifResult {
    /// Rule ID
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    /// Message
    pub message: SarifMessage,
    /// Severity level
    pub level: String,
    /// Locations
    pub locations: Vec<SarifLocation>,
}

/// SARIF message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifMessage {
    /// Message text
    pub text: String,
}

/// SARIF location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifLocation {
    /// Physical location
    #[serde(rename = "physicalLocation")]
    pub physical_location: SarifPhysicalLocation,
}

/// SARIF physical location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    /// Artifact location
    #[serde(rename = "artifactLocation")]
    pub artifact_location: SarifArtifactLocation,
}

/// SARIF artifact location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    /// URI
    pub uri: String,
}

/// SARIF report generator
pub struct SarifReporter;

impl SarifReporter {
    /// Create a new SARIF reporter
    pub fn new() -> Self {
        Self
    }

    /// Generate a SARIF report from security results
    pub fn generate(&self, results: Vec<SecurityResult>) -> Result<String, super::Error> {
        let report = self.build_report(results)?;
        serde_json::to_string_pretty(&report)
            .map_err(|e| super::Error::Serialization(e.to_string()))
    }

    /// Build a SARIF report from security results
    fn build_report(&self, results: Vec<SecurityResult>) -> Result<SarifReport, super::Error> {
        let runs: Vec<SarifRun> = results
            .into_iter()
            .map(|result| self.build_run(result))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SarifReport {
            schema: SARIF_SCHEMA.to_string(),
            version: SARIF_VERSION.to_string(),
            runs,
        })
    }

    /// Build a SARIF run from a security result
    fn build_run(&self, result: SecurityResult) -> Result<SarifRun, super::Error> {
        // Build unique rules map
        let mut rules_map: HashMap<String, SarifRule> = HashMap::new();
        for vuln in &result.vulnerabilities {
            if !rules_map.contains_key(&vuln.id) {
                let rule = SarifRule {
                    id: vuln.id.clone(),
                    name: vuln.id.clone(),
                    short_description: SarifMessage {
                        text: format!("Vulnerability in package {}", vuln.package),
                    },
                    full_description: SarifMessage {
                        text: vuln.description.clone(),
                    },
                    help_uri: Self::build_help_uri(&vuln.id),
                    default_configuration: SarifConfiguration {
                        level: severity_to_level(&vuln.severity).to_string(),
                    },
                };
                rules_map.insert(vuln.id.clone(), rule);
            }
        }

        // Build results
        let sarif_results: Vec<SarifResult> = result
            .vulnerabilities
            .iter()
            .map(|vuln| {
                let message_text = if let Some(ref fixed_version) = vuln.fixed_version {
                    format!(
                        "Package {} version {} is vulnerable. Fixed in {}.",
                        vuln.package, vuln.installed_version, fixed_version
                    )
                } else {
                    format!(
                        "Package {} version {} is vulnerable. No fix available.",
                        vuln.package, vuln.installed_version
                    )
                };

                SarifResult {
                    rule_id: vuln.id.clone(),
                    level: severity_to_level(&vuln.severity).to_string(),
                    message: SarifMessage { text: message_text },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: format!("container/{}", vuln.package),
                            },
                        },
                    }],
                }
            })
            .collect();

        let rules: Vec<SarifRule> = rules_map.into_values().collect();

        Ok(SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "firestream-vib".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    information_uri: Some("https://github.com/Cogent-Creation-Co/Firestream".to_string()),
                    rules,
                },
            },
            results: sarif_results,
        })
    }

    /// Build help URI for a CVE
    fn build_help_uri(cve_id: &str) -> Option<String> {
        if cve_id.starts_with("CVE-") {
            Some(format!("https://nvd.nist.gov/vuln/detail/{}", cve_id))
        } else {
            None
        }
    }

    /// Write report to a file
    pub fn write_to_file(&self, results: Vec<SecurityResult>, path: &str) -> Result<(), super::Error> {
        let sarif = self.generate(results)?;
        std::fs::write(path, sarif)?;
        Ok(())
    }
}

impl Default for SarifReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Map Severity to SARIF level
fn severity_to_level(severity: &Severity) -> &'static str {
    match severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low | Severity::Unknown => "note",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::security::Vulnerability;
    use std::collections::HashMap;

    #[test]
    fn test_severity_to_level() {
        assert_eq!(severity_to_level(&Severity::Critical), "error");
        assert_eq!(severity_to_level(&Severity::High), "error");
        assert_eq!(severity_to_level(&Severity::Medium), "warning");
        assert_eq!(severity_to_level(&Severity::Low), "note");
        assert_eq!(severity_to_level(&Severity::Unknown), "note");
    }

    #[test]
    fn test_build_help_uri() {
        assert_eq!(
            SarifReporter::build_help_uri("CVE-2023-12345"),
            Some("https://nvd.nist.gov/vuln/detail/CVE-2023-12345".to_string())
        );
        assert_eq!(
            SarifReporter::build_help_uri("GHSA-1234-5678-9012"),
            None
        );
    }

    #[test]
    fn test_sarif_report_generation() {
        let reporter = SarifReporter::new();

        let mut summary = HashMap::new();
        summary.insert("CRITICAL".to_string(), 1);
        summary.insert("HIGH".to_string(), 1);

        let security_result = SecurityResult {
            scanner: "trivy".to_string(),
            timestamp: "2025-12-21T00:00:00Z".to_string(),
            vulnerabilities: vec![
                Vulnerability {
                    id: "CVE-2023-12345".to_string(),
                    severity: Severity::Critical,
                    package: "openssl".to_string(),
                    installed_version: "1.0.0".to_string(),
                    fixed_version: Some("1.1.0".to_string()),
                    description: "Critical vulnerability in OpenSSL".to_string(),
                },
                Vulnerability {
                    id: "CVE-2023-54321".to_string(),
                    severity: Severity::High,
                    package: "libcurl".to_string(),
                    installed_version: "7.0.0".to_string(),
                    fixed_version: None,
                    description: "High severity vulnerability in libcurl".to_string(),
                },
            ],
            summary,
        };

        let result = reporter.generate(vec![security_result]);
        assert!(result.is_ok());

        let sarif_json = result.unwrap();
        assert!(sarif_json.contains("$schema"));
        assert!(sarif_json.contains(SARIF_SCHEMA));
        assert!(sarif_json.contains("2.1.0"));
        assert!(sarif_json.contains("firestream-vib"));
        assert!(sarif_json.contains("CVE-2023-12345"));
        assert!(sarif_json.contains("CVE-2023-54321"));
        assert!(sarif_json.contains("openssl"));
        assert!(sarif_json.contains("libcurl"));
        assert!(sarif_json.contains("https://nvd.nist.gov/vuln/detail/CVE-2023-12345"));
    }

    #[test]
    fn test_empty_sarif_report() {
        let reporter = SarifReporter::new();
        let result = reporter.generate(vec![]);
        assert!(result.is_ok());

        let sarif_json = result.unwrap();
        assert!(sarif_json.contains("$schema"));
        assert!(sarif_json.contains("runs"));
    }

    #[test]
    fn test_sarif_report_structure() {
        let reporter = SarifReporter::new();

        let mut summary = HashMap::new();
        summary.insert("MEDIUM".to_string(), 1);

        let security_result = SecurityResult {
            scanner: "grype".to_string(),
            timestamp: "2025-12-21T00:00:00Z".to_string(),
            vulnerabilities: vec![
                Vulnerability {
                    id: "CVE-2023-99999".to_string(),
                    severity: Severity::Medium,
                    package: "test-package".to_string(),
                    installed_version: "1.0.0".to_string(),
                    fixed_version: Some("2.0.0".to_string()),
                    description: "Test vulnerability".to_string(),
                },
            ],
            summary,
        };

        let report = reporter.build_report(vec![security_result]).unwrap();

        assert_eq!(report.schema, SARIF_SCHEMA);
        assert_eq!(report.version, SARIF_VERSION);
        assert_eq!(report.runs.len(), 1);

        let run = &report.runs[0];
        assert_eq!(run.tool.driver.name, "firestream-vib");
        assert_eq!(run.tool.driver.rules.len(), 1);
        assert_eq!(run.results.len(), 1);

        let rule = &run.tool.driver.rules[0];
        assert_eq!(rule.id, "CVE-2023-99999");
        assert_eq!(rule.default_configuration.level, "warning");

        let result = &run.results[0];
        assert_eq!(result.rule_id, "CVE-2023-99999");
        assert_eq!(result.level, "warning");
        assert!(result.message.text.contains("test-package"));
        assert!(result.message.text.contains("1.0.0"));
        assert!(result.message.text.contains("2.0.0"));
    }
}

//! JSON report generation
//!
//! Generates test reports in JSON format.

use crate::runner::TestResult;
use serde::{Deserialize, Serialize};

/// JSON test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonReport {
    /// Report version
    pub version: String,
    /// Test suite name
    pub suite: String,
    /// Timestamp
    pub timestamp: String,
    /// Total tests
    pub total: usize,
    /// Passed tests
    pub passed: usize,
    /// Failed tests
    pub failed: usize,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Individual test results
    pub results: Vec<TestResult>,
}

/// JSON report generator
pub struct JsonReporter;

impl JsonReporter {
    /// Create a new JSON reporter
    pub fn new() -> Self {
        Self
    }

    /// Generate a JSON report from test results
    pub fn generate(&self, suite_name: &str, results: Vec<TestResult>) -> Result<String, super::Error> {
        let total = results.len();
        let passed = results.iter().filter(|r| r.success).count();
        let failed = total - passed;
        let duration_ms = results.iter().map(|r| r.duration_ms).sum();

        let report = JsonReport {
            version: "1.0".to_string(),
            suite: suite_name.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            total,
            passed,
            failed,
            duration_ms,
            results,
        };

        serde_json::to_string_pretty(&report)
            .map_err(|e| super::Error::Serialization(e.to_string()))
    }

    /// Write report to a file
    pub fn write_to_file(&self, suite_name: &str, results: Vec<TestResult>, path: &str) -> Result<(), super::Error> {
        let json = self.generate(suite_name, results)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for JsonReporter {
    fn default() -> Self {
        Self::new()
    }
}

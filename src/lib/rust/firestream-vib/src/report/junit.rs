//! JUnit XML report generation
//!
//! Generates test reports in JUnit XML format for CI/CD integration.

use crate::runner::TestResult;

/// JUnit report generator
pub struct JUnitReporter;

impl JUnitReporter {
    /// Create a new JUnit reporter
    pub fn new() -> Self {
        Self
    }

    /// Generate a JUnit XML report from test results
    pub fn generate(&self, suite_name: &str, results: Vec<TestResult>) -> Result<String, super::Error> {
        let total = results.len();
        let failures = results.iter().filter(|r| !r.success).count();
        let duration_secs: f64 = results.iter().map(|r| r.duration_ms as f64 / 1000.0).sum();

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">\n",
            suite_name, total, failures, duration_secs
        ));

        for result in results {
            let duration_secs = result.duration_ms as f64 / 1000.0;
            xml.push_str(&format!(
                "  <testcase name=\"{}\" time=\"{:.3}\"",
                result.name, duration_secs
            ));

            if result.success {
                xml.push_str(" />\n");
            } else {
                xml.push_str(">\n");
                xml.push_str(&format!(
                    "    <failure message=\"{}\">{}</failure>\n",
                    result.error.as_deref().unwrap_or("Test failed"),
                    escape_xml(&result.output)
                ));
                xml.push_str("  </testcase>\n");
            }
        }

        xml.push_str("</testsuite>\n");
        Ok(xml)
    }

    /// Write report to a file
    pub fn write_to_file(&self, suite_name: &str, results: Vec<TestResult>, path: &str) -> Result<(), super::Error> {
        let xml = self.generate(suite_name, results)?;
        std::fs::write(path, xml)?;
        Ok(())
    }
}

impl Default for JUnitReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

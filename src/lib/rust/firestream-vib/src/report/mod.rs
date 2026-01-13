//! Test report generation
//!
//! Generates test reports in various formats (JSON, JUnit, SARIF).

pub mod json;
pub mod junit;
pub mod sarif;

pub use json::JsonReporter;
pub use junit::JUnitReporter;
pub use sarif::SarifReporter;

/// Error type for report operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Report generation failed: {0}")]
    GenerationFailed(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    JUnit,
    Sarif,
}

impl std::str::FromStr for ReportFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "junit" => Ok(Self::JUnit),
            "sarif" => Ok(Self::Sarif),
            _ => Err(Error::InvalidFormat(s.to_string())),
        }
    }
}

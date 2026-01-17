//! Error types for templatizer
//!
//! This module provides a unified error type for all template generation operations
//! across Spark, Puppeteer, and Superset generators.

use std::path::PathBuf;

/// Custom error type for templatizer operations
#[derive(Debug, thiserror::Error)]
pub enum TemplatizerError {
    /// Template not found in embedded assets
    #[error("Template not found: {name}")]
    TemplateNotFound { name: String },

    /// Failed to render template
    #[error("Failed to render template '{template}': {message}")]
    RenderFailed { template: String, message: String },

    /// Invalid configuration provided
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    /// Output directory operation failed
    #[error("Output directory error at {}: {message}", path.display())]
    OutputError { path: PathBuf, message: String },

    /// Template extraction failed
    #[error("Failed to extract templates: {0}")]
    ExtractionFailed(String),

    /// IO operation failed
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Tera template engine error
    #[error("Tera error: {0}")]
    TeraError(#[from] tera::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// YAML serialization/deserialization error
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// HTTP request error (for Superset uploads)
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// ZIP archive error (for Superset exports)
    #[error("ZIP error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    /// Formatting error
    #[error("Format error: {0}")]
    FmtError(#[from] std::fmt::Error),

    /// Generic error for other cases
    #[error("{0}")]
    Other(String),
}

/// Result type alias using TemplatizerError
pub type Result<T> = std::result::Result<T, TemplatizerError>;

impl TemplatizerError {
    /// Create a template not found error
    pub fn template_not_found(name: impl Into<String>) -> Self {
        Self::TemplateNotFound { name: name.into() }
    }

    /// Create a render failed error
    pub fn render_failed(template: impl Into<String>, message: impl Into<String>) -> Self {
        Self::RenderFailed {
            template: template.into(),
            message: message.into(),
        }
    }

    /// Create an invalid config error
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Create an output error
    pub fn output_error(path: PathBuf, message: impl Into<String>) -> Self {
        Self::OutputError {
            path,
            message: message.into(),
        }
    }

    /// Create an other error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_not_found_error() {
        let err = TemplatizerError::template_not_found("test.tera");
        assert!(err.to_string().contains("test.tera"));
    }

    #[test]
    fn test_render_failed_error() {
        let err = TemplatizerError::render_failed("base.tera", "missing variable");
        assert!(err.to_string().contains("base.tera"));
        assert!(err.to_string().contains("missing variable"));
    }

    #[test]
    fn test_invalid_config_error() {
        let err = TemplatizerError::invalid_config("project_name cannot be empty");
        assert!(err.to_string().contains("project_name cannot be empty"));
    }

    #[test]
    fn test_output_error() {
        let err = TemplatizerError::output_error(
            PathBuf::from("/tmp/output"),
            "permission denied",
        );
        assert!(err.to_string().contains("/tmp/output"));
        assert!(err.to_string().contains("permission denied"));
    }
}

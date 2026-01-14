//! Embedded template management
//!
//! This module provides access to the template files embedded at compile time.
//! Templates are organized by generator type: puppeteer, spark, and superset.

use crate::error::{Result, TemplatizerError};
use rust_embed::Embed;
use std::path::PathBuf;
use tracing::info;

/// Embedded template assets
///
/// This struct uses rust-embed to embed all template files from the
/// `embedded/` directory (populated by build.rs using workspace-embed).
#[derive(Embed)]
#[folder = "embedded/"]
pub struct EmbeddedTemplates;

/// Re-export ExtractedWorkspace from workspace-embed for convenience
pub use workspace_embed::ExtractedWorkspace;

/// Template type identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    /// Puppeteer web scraper templates
    Puppeteer,
    /// Spark application templates (Python and Scala)
    Spark,
    /// Superset dashboard templates
    Superset,
}

impl TemplateType {
    /// Get the directory prefix for this template type
    pub fn prefix(&self) -> &'static str {
        match self {
            TemplateType::Puppeteer => "src/templates/puppeteer",
            TemplateType::Spark => "src/templates/spark",
            TemplateType::Superset => "src/templates/superset",
        }
    }

    /// Get the short name for this template type
    pub fn name(&self) -> &'static str {
        match self {
            TemplateType::Puppeteer => "puppeteer",
            TemplateType::Spark => "spark",
            TemplateType::Superset => "superset",
        }
    }
}

/// Extract the embedded templates to a temporary directory
///
/// Returns a handle to the extracted workspace. The workspace is automatically
/// cleaned up when the handle is dropped.
pub fn extract_templates() -> Result<ExtractedWorkspace> {
    info!("Extracting embedded templates...");

    let workspace = workspace_embed::ExtractedWorkspace::extract::<EmbeddedTemplates>()
        .map_err(|e| TemplatizerError::ExtractionFailed(format!("Failed to extract templates: {}", e)))?;

    info!(
        "Extracted {} files to {}",
        workspace.file_count(),
        workspace.root().display()
    );

    Ok(workspace)
}

/// Get the path to templates of a specific type within an extracted workspace
pub fn templates_dir(workspace: &ExtractedWorkspace, template_type: TemplateType) -> PathBuf {
    workspace.path(template_type.prefix())
}

/// List all available templates of a specific type
pub fn list_templates(workspace: &ExtractedWorkspace, template_type: TemplateType) -> Result<Vec<String>> {
    let mut templates = Vec::new();
    let templates_path = templates_dir(workspace, template_type);

    if !templates_path.exists() {
        return Ok(templates);
    }

    for entry in std::fs::read_dir(&templates_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                templates.push(name.to_string());
            }
        } else if path.is_file() {
            // For superset, templates are files not directories
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".tera") {
                    templates.push(name.to_string());
                }
            }
        }
    }

    templates.sort();
    Ok(templates)
}

/// Check if a specific template exists
pub fn has_template(workspace: &ExtractedWorkspace, template_type: TemplateType, name: &str) -> bool {
    let template_path = templates_dir(workspace, template_type).join(name);
    template_path.exists()
}

/// Get the path to a specific template
pub fn template_path(workspace: &ExtractedWorkspace, template_type: TemplateType, name: &str) -> Option<PathBuf> {
    let path = templates_dir(workspace, template_type).join(name);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Get count of embedded files
pub fn embedded_file_count() -> usize {
    EmbeddedTemplates::iter().count()
}

/// List all embedded file paths
pub fn list_embedded_files() -> Vec<String> {
    EmbeddedTemplates::iter().map(|s| s.to_string()).collect()
}

/// List embedded files filtered by template type
pub fn list_embedded_files_for_type(template_type: TemplateType) -> Vec<String> {
    let prefix = template_type.prefix();
    EmbeddedTemplates::iter()
        .filter(|s| s.starts_with(prefix))
        .map(|s| s.to_string())
        .collect()
}

/// Get the content of an embedded file directly (without extraction)
pub fn get_embedded_file(path: &str) -> Option<Vec<u8>> {
    EmbeddedTemplates::get(path).map(|f| f.data.to_vec())
}

/// Get the content of an embedded file as a string
pub fn get_embedded_file_str(path: &str) -> Option<String> {
    get_embedded_file(path).and_then(|bytes| String::from_utf8(bytes).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_type_prefix() {
        assert_eq!(TemplateType::Puppeteer.prefix(), "src/templates/puppeteer");
        assert_eq!(TemplateType::Spark.prefix(), "src/templates/spark");
        assert_eq!(TemplateType::Superset.prefix(), "src/templates/superset");
    }

    #[test]
    fn test_template_type_name() {
        assert_eq!(TemplateType::Puppeteer.name(), "puppeteer");
        assert_eq!(TemplateType::Spark.name(), "spark");
        assert_eq!(TemplateType::Superset.name(), "superset");
    }

    #[test]
    fn test_embedded_file_count() {
        // Should have at least some embedded files after build
        let count = embedded_file_count();
        // After build, we should have embedded templates
        // This test mainly ensures the function doesn't panic
        assert!(count > 0, "Expected embedded files after build");
    }

    #[test]
    fn test_list_embedded_files() {
        let files = list_embedded_files();
        // After build, we should have embedded templates
        assert!(!files.is_empty(), "Expected embedded files list to be non-empty");
    }
}

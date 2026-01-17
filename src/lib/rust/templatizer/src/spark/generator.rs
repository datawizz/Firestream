//! Generator for Spark application templates
//!
//! This module generates complete Spark applications in both Python and Scala
//! based on configuration.

use crate::config::{GenerationOptions, TemplateConfig};
use crate::embedded::TemplateType;
use crate::engine::TemplateEngine;
use crate::error::{Result, TemplatizerError};
use crate::spark::config::{LanguageType, SparkAppConfig};
use std::path::Path;
use tracing::{debug, info};

/// Generator for Spark applications
pub struct SparkGenerator {
    engine: TemplateEngine,
}

impl SparkGenerator {
    /// Create a new Spark generator
    pub fn new() -> Result<Self> {
        let engine = TemplateEngine::new(TemplateType::Spark)?;
        Ok(Self { engine })
    }

    /// Generate a Spark project to a directory
    pub fn generate(&self, config: &SparkAppConfig, output_dir: &Path) -> Result<()> {
        self.generate_with_options(config, output_dir, &GenerationOptions::default())
    }

    /// Generate with specific options
    pub fn generate_with_options(
        &self,
        config: &SparkAppConfig,
        output_dir: &Path,
        options: &GenerationOptions,
    ) -> Result<()> {
        // Validate config
        config.validate()?;

        info!(
            "Generating Spark {} application: {}",
            config.language, config.app_name
        );

        if options.dry_run {
            info!("Dry run - would generate to: {}", output_dir.display());
            return Ok(());
        }

        // Create output directory
        std::fs::create_dir_all(output_dir).map_err(|e| {
            TemplatizerError::output_error(output_dir.to_path_buf(), e.to_string())
        })?;

        // Convert config to context
        let context = config.to_context()?;

        // Determine template subdirectory based on language
        let template_subdir = config.language.template_dir();

        // Render templates
        for template_name in self.engine.list_templates() {
            // Skip templates not for this language
            if !template_name.contains(template_subdir) {
                continue;
            }

            // Determine output path (remove template prefix and .tera extension)
            let output_file = template_name
                .trim_end_matches(".tera")
                .replace(&format!("{}/", template_subdir), "");

            let output_path = output_dir.join(&output_file);

            // Check for existing file
            if output_path.exists() && !options.overwrite {
                debug!("Skipping existing file: {}", output_path.display());
                continue;
            }

            // Render and write
            self.engine
                .render_to_file(&template_name, &context, &output_path)?;

            if options.verbose {
                info!("Generated: {}", output_path.display());
            }
        }

        info!(
            "Successfully generated Spark application to {}",
            output_dir.display()
        );
        Ok(())
    }

    /// List available templates for a language
    pub fn list_templates(&self, language: LanguageType) -> Vec<String> {
        let template_subdir = language.template_dir();
        self.engine
            .list_templates()
            .into_iter()
            .filter(|t| t.contains(template_subdir))
            .collect()
    }

    /// Verify templates are properly loaded for a language
    pub fn verify_templates(&self, language: LanguageType) -> Result<()> {
        let templates = self.list_templates(language);

        if templates.is_empty() {
            return Err(TemplatizerError::template_not_found(format!(
                "No {} templates found",
                language
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spark::config::{get_python_example, get_scala_example};

    // Note: These tests require the template engine to be properly initialized
    // which depends on embedded templates being available at build time.

    #[test]
    fn test_language_template_dir() {
        assert_eq!(LanguageType::Python.template_dir(), "python");
        assert_eq!(LanguageType::Scala.template_dir(), "scala");
    }

    #[test]
    fn test_python_example_config() {
        let config = get_python_example();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_scala_example_config() {
        let config = get_scala_example();
        assert!(config.validate().is_ok());
    }
}

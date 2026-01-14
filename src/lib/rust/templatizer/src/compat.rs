//! Compatibility layer for legacy templatizer APIs
//!
//! This module provides backwards-compatible types and functions that
//! mirror the legacy templatizer-spark, templatizer-puppeteer, and
//! templatizer-superset crate APIs.
//!
//! This allows existing code (like firestream-tui) to continue working
//! with minimal changes while using the new unified templatizer crate.
//!
//! # Migration
//!
//! To migrate from legacy crates:
//!
//! ```rust,ignore
//! // Old (legacy crate):
//! use templatizer_spark::{SparkTemplatizer, TemplateType};
//!
//! // New (unified crate with compat layer):
//! use templatizer::compat::spark::{SparkTemplatizer, TemplateType};
//! ```

pub mod spark {
    //! Compatibility layer for templatizer-spark
    //!
    //! Provides `SparkTemplatizer` and `TemplateType` that match the legacy API.

    use crate::embedded::TemplateType as UnifiedTemplateType;
    use crate::engine::TemplateEngine;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use tera::Context;

    /// Template type for generation (legacy API compatibility)
    #[derive(Debug, Clone, Copy)]
    pub enum TemplateType {
        Scala,
        Python,
    }

    impl TemplateType {
        /// Get the template type as string
        pub fn as_str(&self) -> &'static str {
            match self {
                TemplateType::Scala => "scala",
                TemplateType::Python => "python",
            }
        }

        /// Convert to subdirectory name for templates
        pub fn subdir(&self) -> &'static str {
            match self {
                TemplateType::Python => "python",
                TemplateType::Scala => "scala",
            }
        }
    }

    impl std::str::FromStr for TemplateType {
        type Err = String;

        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "scala" => Ok(TemplateType::Scala),
                "python" => Ok(TemplateType::Python),
                _ => Err(format!(
                    "Invalid template type: {}. Use 'scala' or 'python'",
                    s
                )),
            }
        }
    }

    /// Legacy-compatible Spark template generator
    ///
    /// This wraps the new TemplateEngine to provide the same API as
    /// the old templatizer-spark crate.
    pub struct SparkTemplatizer {
        template_type: TemplateType,
        engine: TemplateEngine,
        /// Map of template names to their content
        templates: HashMap<String, String>,
    }

    impl SparkTemplatizer {
        /// Create a new templatizer for the specified template type
        pub fn new(template_type: TemplateType) -> std::result::Result<Self, tera::Error> {
            let engine = TemplateEngine::new(UnifiedTemplateType::Spark).map_err(|e| {
                tera::Error::msg(format!("Failed to initialize template engine: {}", e))
            })?;

            // Build the template map for this specific template type (Python or Scala)
            let subdir = template_type.subdir();
            let mut templates = HashMap::new();

            for name in engine.list_templates() {
                // Only include templates for the requested language
                if name.contains(subdir) {
                    templates.insert(name.clone(), name);
                }
            }

            if templates.is_empty() {
                return Err(tera::Error::msg(format!(
                    "No templates found for {:?}. Make sure templates are in the correct directory.",
                    template_type
                )));
            }

            Ok(Self {
                template_type,
                engine,
                templates,
            })
        }

        /// Render all templates to the specified directory
        pub fn render_to_directory(
            &mut self,
            context: Context,
            output_dir: &Path,
        ) -> std::result::Result<(), Box<dyn std::error::Error>> {
            // Create output directory
            fs::create_dir_all(output_dir)?;

            let subdir = self.template_type.subdir();

            // Render each template
            for template_name in self.templates.keys() {
                // Render template
                let rendered = self.engine.render(template_name, &context)?;

                // Determine output path
                // Remove .tera extension and language subdirectory prefix
                let output_file = template_name
                    .trim_end_matches(".tera")
                    .replace(&format!("{}/", subdir), "");

                let output_path = output_dir.join(&output_file);

                // Create parent directories if needed
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Write file
                fs::write(&output_path, rendered)?;
            }

            Ok(())
        }

        /// Check if templates are properly loaded
        pub fn verify_templates(&self) -> std::result::Result<(), String> {
            if self.templates.is_empty() {
                return Err("No templates loaded".to_string());
            }
            Ok(())
        }

        /// Get a list of all template files
        pub fn list_templates(&self) -> Vec<&str> {
            self.templates.keys().map(|s| s.as_str()).collect()
        }

        /// Render a single template
        pub fn render_template(
            &self,
            template_name: &str,
            context: &Context,
        ) -> std::result::Result<String, tera::Error> {
            self.engine
                .render(template_name, context)
                .map_err(|e| tera::Error::msg(e.to_string()))
        }
    }

    /// Configuration builder for Spark applications (legacy API compatibility)
    pub struct SparkAppConfig {
        config: serde_json::Map<String, serde_json::Value>,
    }

    impl SparkAppConfig {
        /// Create a new configuration builder
        pub fn new(app_name: &str, version: &str) -> Self {
            let mut config = serde_json::Map::new();
            config.insert("app_name".to_string(), serde_json::json!(app_name));
            config.insert("version".to_string(), serde_json::json!(version));

            Self { config }
        }

        /// Set organization
        pub fn organization(mut self, org: &str) -> Self {
            self.config
                .insert("organization".to_string(), serde_json::json!(org));
            self
        }

        /// Set Docker registry
        pub fn docker_registry(mut self, registry: &str) -> Self {
            self.config
                .insert("docker_registry".to_string(), serde_json::json!(registry));
            self
        }

        /// Enable S3 support
        pub fn enable_s3(mut self) -> Self {
            self.config
                .insert("s3_enabled".to_string(), serde_json::json!(true));
            self
        }

        /// Enable Delta Lake support
        pub fn enable_delta(mut self) -> Self {
            self.config
                .insert("delta_enabled".to_string(), serde_json::json!(true));
            self
        }

        /// Set Kubernetes namespace
        pub fn namespace(mut self, ns: &str) -> Self {
            self.config
                .insert("namespace".to_string(), serde_json::json!(ns));
            self
        }

        /// Build the configuration into a Tera context
        pub fn build(self) -> std::result::Result<Context, tera::Error> {
            Context::from_serialize(serde_json::Value::Object(self.config))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::spark::*;

    #[test]
    fn test_template_type_from_str() {
        assert!(matches!(
            "scala".parse::<TemplateType>(),
            Ok(TemplateType::Scala)
        ));
        assert!(matches!(
            "python".parse::<TemplateType>(),
            Ok(TemplateType::Python)
        ));
        assert!(matches!(
            "SCALA".parse::<TemplateType>(),
            Ok(TemplateType::Scala)
        ));
        assert!("java".parse::<TemplateType>().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = SparkAppConfig::new("Test App", "1.0.0")
            .organization("com.example")
            .docker_registry("example.io")
            .enable_s3()
            .build()
            .unwrap();

        assert_eq!(config.get("app_name").unwrap().as_str().unwrap(), "Test App");
        assert_eq!(config.get("s3_enabled").unwrap().as_bool().unwrap(), true);
    }
}

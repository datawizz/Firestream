//! Unified Tera template engine wrapper
//!
//! This module provides a unified Tera engine configuration used by all
//! template generators (Spark, Puppeteer, Superset).

use crate::embedded::{self, ExtractedWorkspace, TemplateType};
use crate::error::{Result, TemplatizerError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::{Context, Tera, Value};
use tracing::{debug, info};

/// Unified template engine wrapper
///
/// Provides a consistent interface for rendering templates across all
/// generator types with common configuration and helper functions.
pub struct TemplateEngine {
    /// The Tera template engine instance
    tera: Tera,
    /// The extracted workspace containing templates
    workspace: ExtractedWorkspace,
    /// The type of templates this engine is configured for
    template_type: TemplateType,
}

impl TemplateEngine {
    /// Create a new template engine for the specified template type
    ///
    /// This extracts the embedded templates and initializes Tera with
    /// all templates of the specified type.
    pub fn new(template_type: TemplateType) -> Result<Self> {
        let workspace = embedded::extract_templates()?;
        Self::from_workspace(workspace, template_type)
    }

    /// Create a template engine from an existing extracted workspace
    ///
    /// Useful when you need to share a workspace across multiple engines.
    pub fn from_workspace(workspace: ExtractedWorkspace, template_type: TemplateType) -> Result<Self> {
        let templates_dir = embedded::templates_dir(&workspace, template_type);

        // Build glob pattern for Tera
        let glob_pattern = format!("{}/**/*.tera", templates_dir.display());

        info!(
            "Initializing Tera engine for {} templates from {}",
            template_type.name(),
            templates_dir.display()
        );

        let mut tera = Tera::new(&glob_pattern).map_err(|e| {
            TemplatizerError::other(format!(
                "Failed to initialize Tera for {}: {}",
                template_type.name(),
                e
            ))
        })?;

        // Register common filters and functions
        Self::register_helpers(&mut tera);

        debug!(
            "Loaded {} templates for {}",
            tera.get_template_names().count(),
            template_type.name()
        );

        Ok(Self {
            tera,
            workspace,
            template_type,
        })
    }

    /// Register common helper functions and filters for all template types
    fn register_helpers(tera: &mut Tera) {
        // Add snake_case filter
        tera.register_filter("snake_case", |value: &Value, _: &HashMap<String, Value>| {
            let s = value
                .as_str()
                .ok_or_else(|| tera::Error::msg("snake_case filter requires a string"))?;
            Ok(Value::String(to_snake_case(s)))
        });

        // Add camelCase filter
        tera.register_filter("camel_case", |value: &Value, _: &HashMap<String, Value>| {
            let s = value
                .as_str()
                .ok_or_else(|| tera::Error::msg("camel_case filter requires a string"))?;
            Ok(Value::String(to_camel_case(s)))
        });

        // Add PascalCase filter
        tera.register_filter("pascal_case", |value: &Value, _: &HashMap<String, Value>| {
            let s = value
                .as_str()
                .ok_or_else(|| tera::Error::msg("pascal_case filter requires a string"))?;
            Ok(Value::String(to_pascal_case(s)))
        });

        // Add kebab-case filter
        tera.register_filter("kebab_case", |value: &Value, _: &HashMap<String, Value>| {
            let s = value
                .as_str()
                .ok_or_else(|| tera::Error::msg("kebab_case filter requires a string"))?;
            Ok(Value::String(to_kebab_case(s)))
        });
    }

    /// Get a reference to the underlying Tera instance
    pub fn tera(&self) -> &Tera {
        &self.tera
    }

    /// Get a mutable reference to the underlying Tera instance
    pub fn tera_mut(&mut self) -> &mut Tera {
        &mut self.tera
    }

    /// Get the template type this engine is configured for
    pub fn template_type(&self) -> TemplateType {
        self.template_type
    }

    /// Get a reference to the extracted workspace
    pub fn workspace(&self) -> &ExtractedWorkspace {
        &self.workspace
    }

    /// Get the root path of the templates for this engine
    pub fn templates_root(&self) -> PathBuf {
        embedded::templates_dir(&self.workspace, self.template_type)
    }

    /// List all available template names
    pub fn list_templates(&self) -> Vec<String> {
        self.tera
            .get_template_names()
            .map(|s| s.to_string())
            .collect()
    }

    /// Render a template with the given context
    pub fn render(&self, template_name: &str, context: &Context) -> Result<String> {
        self.tera.render(template_name, context).map_err(|e| {
            TemplatizerError::render_failed(template_name, e.to_string())
        })
    }

    /// Render a template string directly (not from file)
    pub fn render_str(&mut self, template_str: &str, context: &Context) -> Result<String> {
        self.tera.render_str(template_str, context).map_err(|e| {
            TemplatizerError::render_failed("<inline>", e.to_string())
        })
    }

    /// Render a template and write the output to a file
    pub fn render_to_file(
        &self,
        template_name: &str,
        context: &Context,
        output_path: &Path,
    ) -> Result<()> {
        let content = self.render(template_name, context)?;

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TemplatizerError::output_error(parent.to_path_buf(), e.to_string())
            })?;
        }

        std::fs::write(output_path, content).map_err(|e| {
            TemplatizerError::output_error(output_path.to_path_buf(), e.to_string())
        })?;

        debug!("Rendered {} to {}", template_name, output_path.display());
        Ok(())
    }
}

// Helper functions for case conversion

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lower = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if prev_lower || (i > 0 && !result.ends_with('_')) {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_lower = false;
        } else if c == '-' || c == ' ' {
            result.push('_');
            prev_lower = false;
        } else {
            result.push(c);
            prev_lower = c.is_lowercase();
        }
    }

    result
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }

    result
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lower = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if prev_lower || (i > 0 && !result.ends_with('-')) {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_lower = false;
        } else if c == '_' || c == ' ' {
            result.push('-');
            prev_lower = false;
        } else {
            result.push(c);
            prev_lower = c.is_lowercase();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("hello-world"), "hello_world");
        assert_eq!(to_snake_case("hello world"), "hello_world");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("hello-world"), "helloWorld");
        assert_eq!(to_camel_case("hello world"), "helloWorld");
        assert_eq!(to_camel_case("HelloWorld"), "helloWorld");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("hello world"), "HelloWorld");
        assert_eq!(to_pascal_case("helloWorld"), "HelloWorld");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("helloWorld"), "hello-world");
        assert_eq!(to_kebab_case("hello_world"), "hello-world");
        assert_eq!(to_kebab_case("hello world"), "hello-world");
    }
}

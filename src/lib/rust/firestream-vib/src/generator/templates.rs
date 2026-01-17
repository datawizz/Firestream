//! Template engine for generating test files
//!
//! Provides Tera-based templating for Goss and other test specifications.

use std::collections::HashMap;
use tera::Tera;

/// Template engine wrapper
pub struct TemplateEngine {
    /// Tera instance
    tera: Tera,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Result<Self, super::Error> {
        let mut tera = Tera::default();

        // Register default templates (if any)
        // Templates will be loaded from the templates/ directory

        Ok(Self { tera })
    }

    /// Create a template engine from a directory
    pub fn from_dir(dir: &str) -> Result<Self, super::Error> {
        let pattern = format!("{}/**/*", dir);
        let tera = Tera::new(&pattern)?;
        Ok(Self { tera })
    }

    /// Add a template from a string
    pub fn add_template(&mut self, name: &str, content: &str) -> Result<(), super::Error> {
        self.tera.add_raw_template(name, content)?;
        Ok(())
    }

    /// Render a template with variables
    pub fn render(
        &self,
        template_name: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<String, super::Error> {
        let mut tera_context = tera::Context::new();

        for (key, value) in context {
            tera_context.insert(key, value);
        }

        let rendered = self.tera.render(template_name, &tera_context)?;
        Ok(rendered)
    }

    /// Render a template string with variables
    pub fn render_str(
        &mut self,
        template: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<String, super::Error> {
        let template_name = "__inline__";
        self.add_template(template_name, template)?;
        self.render(template_name, context)
    }
}

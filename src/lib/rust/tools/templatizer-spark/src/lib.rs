//! Spark Templatizer - Generate Spark applications from templates
//! 
//! This library provides Tera templates for generating production-ready
//! Spark applications in both Scala and Python, designed for deployment
//! on Kubernetes using the Spark Operator.

pub mod example;

use rust_embed::RustEmbed;
use tera::{Tera, Context};
use std::path::Path;
use std::fs;
use std::collections::HashMap;

/// Embedded Python templates
#[derive(RustEmbed)]
#[folder = "src/templates/spark_python_boilerplate/"]
#[include = "*.tera"]
#[include = "**/*.tera"]
struct PythonTemplates;

/// Embedded Scala templates
#[derive(RustEmbed)]
#[folder = "src/templates/spark_scala_boilerplate/"]
#[include = "*.tera"]
#[include = "**/*.tera"]
struct ScalaTemplates;

/// Template type for generation
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
}

impl std::str::FromStr for TemplateType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "scala" => Ok(TemplateType::Scala),
            "python" => Ok(TemplateType::Python),
            _ => Err(format!("Invalid template type: {}. Use 'scala' or 'python'", s)),
        }
    }
}

/// Main template generator
pub struct SparkTemplatizer {
    template_type: TemplateType,
    tera: Tera,
    /// Map of template names to their content for direct file writing
    templates: HashMap<String, String>,
}

impl SparkTemplatizer {
    /// Create a new templatizer for the specified template type
    pub fn new(template_type: TemplateType) -> Result<Self, tera::Error> {
        let mut tera = Tera::default();
        let mut templates = HashMap::new();
        let mut _template_count = 0;
        
        match template_type {
            TemplateType::Python => {
                for file in PythonTemplates::iter() {
                    let file_path = file.as_ref();
                    if file_path.ends_with(".tera") {
                        _template_count += 1;
                        if let Some(content) = PythonTemplates::get(file_path) {
                            let content_str = std::str::from_utf8(content.data.as_ref())
                                .map_err(|e| tera::Error::msg(format!("Invalid UTF-8 in template {}: {}", file_path, e)))?;
                            
                            // Add to Tera for rendering
                            tera.add_raw_template(file_path, content_str)?;
                            
                            // Store for later use
                            templates.insert(file_path.to_string(), content_str.to_string());
                        }
                    }
                }
            }
            TemplateType::Scala => {
                for file in ScalaTemplates::iter() {
                    let file_path = file.as_ref();
                    if file_path.ends_with(".tera") {
                        if let Some(content) = ScalaTemplates::get(file_path) {
                            let content_str = std::str::from_utf8(content.data.as_ref())
                                .map_err(|e| tera::Error::msg(format!("Invalid UTF-8 in template {}: {}", file_path, e)))?;
                            
                            // Add to Tera for rendering
                            tera.add_raw_template(file_path, content_str)?;
                            
                            // Store for later use
                            templates.insert(file_path.to_string(), content_str.to_string());
                        }
                    }
                }
            }
        }
        
        if templates.is_empty() {
            return Err(tera::Error::msg(format!(
                "No templates found for {:?}. Make sure templates are in the correct directory and the crate was built correctly.",
                template_type
            )));
        }
        
        Ok(Self {
            template_type,
            tera,
            templates,
        })
    }
    
    /// Render all templates to the specified directory
    pub fn render_to_directory(
        &mut self,
        context: Context,
        output_dir: &Path
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create output directory
        fs::create_dir_all(output_dir)?;
        
        // Render each template
        for (template_name, _) in &self.templates {
            
            // Render template
            let rendered = self.tera.render(template_name, &context)?;
            
            // Determine output path (remove .tera extension and any directory prefixes)
            let output_file = template_name
                .trim_end_matches(".tera")
                .trim_start_matches("src/templates/spark_python_boilerplate/")
                .trim_start_matches("src/templates/spark_scala_boilerplate/");
            
            let output_path = output_dir.join(output_file);
            
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
    pub fn verify_templates(&self) -> Result<(), String> {
        if self.templates.is_empty() {
            return Err("No templates loaded".to_string());
        }
        
        // Check Python templates
        let python_count = PythonTemplates::iter().filter(|f| f.ends_with(".tera")).count();
        let scala_count = ScalaTemplates::iter().filter(|f| f.ends_with(".tera")).count();
        
        if python_count == 0 && matches!(self.template_type, TemplateType::Python) {
            return Err("No Python templates found in embedded resources".to_string());
        }
        
        if scala_count == 0 && matches!(self.template_type, TemplateType::Scala) {
            return Err("No Scala templates found in embedded resources".to_string());
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
        context: &Context
    ) -> Result<String, tera::Error> {
        self.tera.render(template_name, context)
    }
}

/// Configuration builder for Spark applications
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
        self.config.insert("organization".to_string(), serde_json::json!(org));
        self
    }
    
    /// Set Docker registry
    pub fn docker_registry(mut self, registry: &str) -> Self {
        self.config.insert("docker_registry".to_string(), serde_json::json!(registry));
        self
    }
    
    /// Enable S3 support
    pub fn enable_s3(mut self) -> Self {
        self.config.insert("s3_enabled".to_string(), serde_json::json!(true));
        self
    }
    
    /// Enable Delta Lake support
    pub fn enable_delta(mut self) -> Self {
        self.config.insert("delta_enabled".to_string(), serde_json::json!(true));
        self
    }
    
    /// Set Kubernetes namespace
    pub fn namespace(mut self, ns: &str) -> Self {
        self.config.insert("namespace".to_string(), serde_json::json!(ns));
        self
    }
    
    /// Build the configuration into a Tera context
    pub fn build(self) -> Result<Context, tera::Error> {
        Context::from_serialize(serde_json::Value::Object(self.config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_type_from_str() {
        assert!(matches!("scala".parse::<TemplateType>(), Ok(TemplateType::Scala)));
        assert!(matches!("python".parse::<TemplateType>(), Ok(TemplateType::Python)));
        assert!(matches!("SCALA".parse::<TemplateType>(), Ok(TemplateType::Scala)));
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
    
    #[test]
    fn test_template_loading() {
        // Test Python templates load correctly
        let python_templatizer = SparkTemplatizer::new(TemplateType::Python).unwrap();
        assert!(!python_templatizer.list_templates().is_empty());
        
        // Test Scala templates load correctly
        let scala_templatizer = SparkTemplatizer::new(TemplateType::Scala).unwrap();
        assert!(!scala_templatizer.list_templates().is_empty());
    }
}

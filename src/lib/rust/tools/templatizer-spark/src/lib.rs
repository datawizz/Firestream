//! Spark Templatizer - Generate Spark applications from templates
//! 
//! This library provides Tera templates for generating production-ready
//! Spark applications in both Scala and Python, designed for deployment
//! on Kubernetes using the Spark Operator.

pub mod example;

use tera::{Tera, Context};
use std::path::Path;
use std::fs;

/// Template type for generation
#[derive(Debug, Clone, Copy)]
pub enum TemplateType {
    Scala,
    Python,
}

impl TemplateType {
    /// Get the template directory path
    pub fn template_path(&self) -> String {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| ".".to_string());
        let base_path = Path::new(&manifest_dir);
        
        let relative_path = match self {
            TemplateType::Scala => "src/templates/spark_scala_boilerplate/**/*.tera",
            TemplateType::Python => "src/templates/spark_python_boilerplate/**/*.tera",
        };
        
        base_path.join(relative_path)
            .to_str()
            .unwrap_or(relative_path)
            .to_string()
    }
    
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
    #[allow(dead_code)]
    template_type: TemplateType,
    tera: Tera,
}

impl SparkTemplatizer {
    /// Create a new templatizer for the specified template type
    pub fn new(template_type: TemplateType) -> Result<Self, tera::Error> {
        let tera = Tera::new(&template_type.template_path())?;
        Ok(Self {
            template_type,
            tera,
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
        for template_name in self.tera.get_template_names() {
            // Skip non-.tera files
            if !template_name.ends_with(".tera") {
                continue;
            }
            
            // Render template
            let rendered = self.tera.render(template_name, &context)?;
            
            // Determine output path (remove .tera extension)
            let output_file = template_name.trim_end_matches(".tera");
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
    
    /// Get a list of all template files
    pub fn list_templates(&self) -> Vec<&str> {
        self.tera.get_template_names().collect()
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
}



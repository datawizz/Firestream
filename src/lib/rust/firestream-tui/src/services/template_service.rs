use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
// use serde::{Deserialize, Serialize};
use tera::Context;
use templatizer_spark::{SparkTemplatizer, TemplateType};
use crate::models::{
    TemplateConfiguration, TemplateVariable, VariableType, 
    VariableGroup, TemplateGenerationRequest, TemplateGenerationResult,
    GeneratedFile, ValidationError
};

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    NotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Tera error: {0}")]
    Tera(#[from] tera::Error),
}

pub struct TemplateService {
    templatizers: HashMap<String, SparkTemplatizer>,
    configurations: HashMap<String, TemplateConfiguration>,
    output_base_path: PathBuf,
}

impl TemplateService {
    pub fn new() -> Result<Self, TemplateError> {
        let mut templatizers = HashMap::new();
        let mut configurations = HashMap::new();
        
        // Initialize Python Spark templatizer
        let python_templatizer = SparkTemplatizer::new(TemplateType::Python)
            .map_err(|e| TemplateError::GenerationFailed(format!("Failed to create Python templatizer: {}", e)))?;
        
        // Verify Python templates loaded
        if let Err(e) = python_templatizer.verify_templates() {
            return Err(TemplateError::GenerationFailed(format!("Python templates verification failed: {}", e)));
        }
        
        templatizers.insert("pyspark".to_string(), python_templatizer);
        
        // Initialize Scala Spark templatizer
        let scala_templatizer = SparkTemplatizer::new(TemplateType::Scala)
            .map_err(|e| TemplateError::GenerationFailed(format!("Failed to create Scala templatizer: {}", e)))?;
        
        // Verify Scala templates loaded
        if let Err(e) = scala_templatizer.verify_templates() {
            return Err(TemplateError::GenerationFailed(format!("Scala templates verification failed: {}", e)));
        }
        
        templatizers.insert("spark-scala".to_string(), scala_templatizer);
        
        // Load configurations
        configurations.insert("pyspark".to_string(), Self::load_python_config()?);
        configurations.insert("spark-scala".to_string(), Self::load_scala_config()?);
        
        // Set output base path - try multiple locations
        let output_base_path = if Path::new("/workspace/out").exists() {
            PathBuf::from("/workspace/out")
        } else if let Ok(current_dir) = std::env::current_dir() {
            current_dir.join("generated")
        } else {
            PathBuf::from("./generated")
        };
        
        Ok(Self {
            templatizers,
            configurations,
            output_base_path,
        })
    }
    
    /// Get template configuration for a specific template type
    pub fn get_configuration(&self, template_type: &str) -> Result<&TemplateConfiguration, TemplateError> {
        self.configurations
            .get(template_type)
            .ok_or_else(|| TemplateError::NotFound(template_type.to_string()))
    }
    
    /// Validate user configuration
    pub fn validate_configuration(
        &self,
        template_type: &str,
        user_values: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<ValidationError>, TemplateError> {
        let config = self.get_configuration(template_type)?;
        let mut errors = Vec::new();
        
        // Check required fields
        for (var_name, var) in &config.variables {
            if var.required && !user_values.contains_key(var_name) {
                errors.push(ValidationError {
                    field: var_name.clone(),
                    message: format!("{} is required", var.name),
                });
            }
        }
        
        // Validate types and constraints
        for (var_name, value) in user_values {
            if let Some(var) = config.variables.get(var_name) {
                if let Err(e) = Self::validate_value(&var.var_type, value) {
                    errors.push(ValidationError {
                        field: var_name.clone(),
                        message: e,
                    });
                }
            }
        }
        
        Ok(errors)
    }
    
    /// Generate template with user configuration
    pub fn generate(
        &mut self,
        request: &TemplateGenerationRequest,
    ) -> Result<TemplateGenerationResult, TemplateError> {
        // First validate configuration and build context
        let validation_errors = self.validate_configuration(&request.template_type, &request.variables)?;
        if !validation_errors.is_empty() {
            return Err(TemplateError::InvalidConfig(
                validation_errors.iter()
                    .map(|e| format!("{}: {}", e.field, e.message))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        
        // Build Tera context from user values
        let context = self.build_context(&request.template_type, &request.variables)?;
        
        // Create output directory - use request path or fallback to base path
        let output_path = if request.output_path.starts_with('/') {
            // Absolute path provided
            PathBuf::from(&request.output_path)
        } else {
            // Relative path - use base path
            self.output_base_path.join(&request.name)
        };
        
        // Try to create the directory
        if let Err(e) = fs::create_dir_all(&output_path) {
            return Err(TemplateError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Failed to create output directory {:?}: {}. Try using a different output path.", output_path, e)
            )));
        }
        
        // Now get the templatizer and generate
        let templatizer = self.templatizers
            .get_mut(&request.template_type)
            .ok_or_else(|| TemplateError::NotFound(request.template_type.clone()))?;
        
        // Generate templates
        templatizer.render_to_directory(context, &output_path)
            .map_err(|e| TemplateError::GenerationFailed(e.to_string()))?;
        
        // Collect generated files
        let generated_files = self.collect_generated_files(&output_path)?;
        
        Ok(TemplateGenerationResult {
            success: true,
            generated_files,
            output_path: output_path.to_string_lossy().to_string(),
            errors: vec![],
        })
    }
    
    /// Build Tera context from user values
    fn build_context(
        &self,
        template_type: &str,
        user_values: &HashMap<String, serde_json::Value>,
    ) -> Result<Context, TemplateError> {
        let config = self.get_configuration(template_type)?;
        let mut context_map = serde_json::Map::new();
        
        // Add user values
        for (key, value) in user_values {
            context_map.insert(key.clone(), value.clone());
        }
        
        // Add defaults for missing optional values
        for (var_name, var) in &config.variables {
            if !context_map.contains_key(var_name) {
                if let Some(default) = &var.default_value {
                    context_map.insert(var_name.clone(), default.clone());
                }
            }
        }
        
        // Add computed values
        self.add_computed_values(&mut context_map, template_type);
        
        // Add missing template variables with defaults
        if !context_map.contains_key("additional_pytest_markers") {
            context_map.insert("additional_pytest_markers".to_string(), serde_json::json!([]));
        }
        
        Context::from_serialize(serde_json::Value::Object(context_map))
            .map_err(|e| TemplateError::GenerationFailed(e.to_string()))
    }
    
    /// Add computed values based on user configuration
    fn add_computed_values(&self, context: &mut serde_json::Map<String, serde_json::Value>, template_type: &str) {
        // Package name from app name
        if let Some(app_name) = context.get("app_name").and_then(|v| v.as_str()) {
            if !context.contains_key("package_name") {
                let package_name = app_name
                    .to_lowercase()
                    .replace('-', "_")
                    .replace(' ', "_");
                context.insert("package_name".to_string(), serde_json::json!(package_name));
            }
        }
        
        // Python major version from full version
        if template_type == "pyspark" {
            if let Some(python_version) = context.get("python_version").and_then(|v| v.as_str()) {
                let major_version = python_version.split('.').next().unwrap_or("3");
                context.insert("python_major_version".to_string(), serde_json::json!(major_version));
            }
        }
        
        // Spark configs for Kubernetes
        if let Some(s3_enabled) = context.get("s3_enabled").and_then(|v| v.as_bool()) {
            if s3_enabled {
                let mut spark_conf = context.get("spark_conf")
                    .and_then(|v| v.as_object())
                    .cloned()
                    .unwrap_or_default();
                
                spark_conf.insert(
                    "spark.hadoop.fs.s3a.access.key".to_string(),
                    serde_json::json!("${S3_ACCESS_KEY}")
                );
                spark_conf.insert(
                    "spark.hadoop.fs.s3a.secret.key".to_string(),
                    serde_json::json!("${S3_SECRET_KEY}")
                );
                
                context.insert("spark_conf".to_string(), serde_json::json!(spark_conf));
            }
        }
    }
    
    /// Validate a value against its type
    fn validate_value(var_type: &VariableType, value: &serde_json::Value) -> Result<(), String> {
        match var_type {
            VariableType::String => {
                if !value.is_string() {
                    return Err("Expected string value".to_string());
                }
            }
            VariableType::Boolean => {
                if !value.is_boolean() {
                    return Err("Expected boolean value".to_string());
                }
            }
            VariableType::Integer => {
                if !value.is_i64() && !value.is_u64() {
                    return Err("Expected integer value".to_string());
                }
            }
            VariableType::Float => {
                if !value.is_f64() && !value.is_i64() {
                    return Err("Expected numeric value".to_string());
                }
            }
            VariableType::Array(inner_type) => {
                if let Some(array) = value.as_array() {
                    for item in array {
                        Self::validate_value(inner_type, item)?;
                    }
                } else {
                    return Err("Expected array value".to_string());
                }
            }
            VariableType::Choice(options) => {
                if let Some(str_val) = value.as_str() {
                    if !options.contains(&str_val.to_string()) {
                        return Err(format!("Value must be one of: {}", options.join(", ")));
                    }
                } else {
                    return Err("Expected string value for choice".to_string());
                }
            }
            VariableType::Object => {
                if !value.is_object() {
                    return Err("Expected object value".to_string());
                }
            }
        }
        Ok(())
    }
    
    /// Collect all generated files
    fn collect_generated_files(&self, path: &Path) -> Result<Vec<GeneratedFile>, TemplateError> {
        let mut files = Vec::new();
        
        if !path.exists() {
            return Ok(files);
        }
        
        self.collect_files_recursive(path, path, &mut files)?;
        Ok(files)
    }
    
    fn collect_files_recursive(
        &self,
        base_path: &Path,
        current_path: &Path,
        files: &mut Vec<GeneratedFile>,
    ) -> Result<(), TemplateError> {
        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let relative_path = path.strip_prefix(base_path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                
                let metadata = fs::metadata(&path)?;
                let content = fs::read_to_string(&path).unwrap_or_else(|_| String::from("[binary file]"));
                
                files.push(GeneratedFile {
                    path: relative_path,
                    content,
                    size: metadata.len(),
                });
            } else if path.is_dir() {
                self.collect_files_recursive(base_path, &path, files)?;
            }
        }
        Ok(())
    }
    
    /// Load Python template configuration
    fn load_python_config() -> Result<TemplateConfiguration, TemplateError> {
        let mut variables = HashMap::new();
        let mut groups = vec![];
        
        // Core Settings Group
        groups.push(VariableGroup {
            name: "core".to_string(),
            display_name: "Core Settings".to_string(),
            description: "Basic application settings".to_string(),
            collapsed: false,
            order: 0,
        });
        
        variables.insert("app_name".to_string(), TemplateVariable {
            name: "App Name".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("My Spark App")),
            description: "Name of your Spark application".to_string(),
            required: true,
            group: "core".to_string(),
            display_order: 0,
        });
        
        variables.insert("organization".to_string(), TemplateVariable {
            name: "Organization".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("com.example")),
            description: "Organization name (e.g., com.example)".to_string(),
            required: true,
            group: "core".to_string(),
            display_order: 1,
        });
        
        variables.insert("version".to_string(), TemplateVariable {
            name: "Version".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("1.0.0")),
            description: "Application version".to_string(),
            required: true,
            group: "core".to_string(),
            display_order: 2,
        });
        
        // Python Configuration Group
        groups.push(VariableGroup {
            name: "python".to_string(),
            display_name: "Python Configuration".to_string(),
            description: "Python-specific settings".to_string(),
            collapsed: true,
            order: 1,
        });
        
        variables.insert("python_version".to_string(), TemplateVariable {
            name: "Python Version".to_string(),
            var_type: VariableType::Choice(vec!["3.8".to_string(), "3.9".to_string(), "3.10".to_string(), "3.11".to_string()]),
            default_value: Some(serde_json::json!("3.10")),
            description: "Python version to use".to_string(),
            required: true,
            group: "python".to_string(),
            display_order: 10,
        });
        
        variables.insert("pyspark_version".to_string(), TemplateVariable {
            name: "PySpark Version".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("3.5.1")),
            description: "PySpark version".to_string(),
            required: true,
            group: "python".to_string(),
            display_order: 11,
        });
        
        // Features Group
        groups.push(VariableGroup {
            name: "features".to_string(),
            display_name: "Features".to_string(),
            description: "Enable/disable features".to_string(),
            collapsed: true,
            order: 2,
        });
        
        variables.insert("s3_enabled".to_string(), TemplateVariable {
            name: "Enable S3".to_string(),
            var_type: VariableType::Boolean,
            default_value: Some(serde_json::json!(false)),
            description: "Include S3/AWS support".to_string(),
            required: false,
            group: "features".to_string(),
            display_order: 20,
        });
        
        variables.insert("delta_enabled".to_string(), TemplateVariable {
            name: "Enable Delta Lake".to_string(),
            var_type: VariableType::Boolean,
            default_value: Some(serde_json::json!(false)),
            description: "Include Delta Lake support".to_string(),
            required: false,
            group: "features".to_string(),
            display_order: 21,
        });
        
        variables.insert("monitoring_enabled".to_string(), TemplateVariable {
            name: "Enable Monitoring".to_string(),
            var_type: VariableType::Boolean,
            default_value: Some(serde_json::json!(true)),
            description: "Include Prometheus monitoring".to_string(),
            required: false,
            group: "features".to_string(),
            display_order: 22,
        });
        
        // Kubernetes Group
        groups.push(VariableGroup {
            name: "kubernetes".to_string(),
            display_name: "Kubernetes".to_string(),
            description: "Kubernetes deployment settings".to_string(),
            collapsed: false,
            order: 3,
        });
        
        variables.insert("namespace".to_string(), TemplateVariable {
            name: "Namespace".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("spark-apps")),
            description: "Kubernetes namespace".to_string(),
            required: true,
            group: "kubernetes".to_string(),
            display_order: 30,
        });
        
        variables.insert("driver_memory".to_string(), TemplateVariable {
            name: "Driver Memory".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("2g")),
            description: "Driver memory allocation".to_string(),
            required: true,
            group: "kubernetes".to_string(),
            display_order: 31,
        });
        
        variables.insert("executor_memory".to_string(), TemplateVariable {
            name: "Executor Memory".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("4g")),
            description: "Executor memory allocation".to_string(),
            required: true,
            group: "kubernetes".to_string(),
            display_order: 32,
        });
        
        variables.insert("executor_instances".to_string(), TemplateVariable {
            name: "Executor Instances".to_string(),
            var_type: VariableType::Integer,
            default_value: Some(serde_json::json!(3)),
            description: "Number of executor instances".to_string(),
            required: true,
            group: "kubernetes".to_string(),
            display_order: 33,
        });
        
        variables.insert("docker_registry".to_string(), TemplateVariable {
            name: "Docker Registry".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("myregistry.io")),
            description: "Docker registry URL".to_string(),
            required: true,
            group: "kubernetes".to_string(),
            display_order: 34,
        });
        
        Ok(TemplateConfiguration {
            variables,
            variable_groups: groups,
        })
    }
    
    /// Load Scala template configuration
    fn load_scala_config() -> Result<TemplateConfiguration, TemplateError> {
        // Similar to Python config but with Scala-specific settings
        let mut variables = HashMap::new();
        let mut groups = vec![];
        
        // Core Settings Group
        groups.push(VariableGroup {
            name: "core".to_string(),
            display_name: "Core Settings".to_string(),
            description: "Basic application settings".to_string(),
            collapsed: false,
            order: 0,
        });
        
        variables.insert("app_name".to_string(), TemplateVariable {
            name: "App Name".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("My Spark App")),
            description: "Name of your Spark application".to_string(),
            required: true,
            group: "core".to_string(),
            display_order: 0,
        });
        
        variables.insert("organization".to_string(), TemplateVariable {
            name: "Organization".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("com.example")),
            description: "Organization name (e.g., com.example)".to_string(),
            required: true,
            group: "core".to_string(),
            display_order: 1,
        });
        
        variables.insert("version".to_string(), TemplateVariable {
            name: "Version".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("1.0.0")),
            description: "Application version".to_string(),
            required: true,
            group: "core".to_string(),
            display_order: 2,
        });
        
        // Scala Configuration Group
        groups.push(VariableGroup {
            name: "scala".to_string(),
            display_name: "Scala Configuration".to_string(),
            description: "Scala-specific settings".to_string(),
            collapsed: true,
            order: 1,
        });
        
        variables.insert("scala_version".to_string(), TemplateVariable {
            name: "Scala Version".to_string(),
            var_type: VariableType::Choice(vec!["2.12".to_string(), "2.13".to_string()]),
            default_value: Some(serde_json::json!("2.12")),
            description: "Scala version".to_string(),
            required: true,
            group: "scala".to_string(),
            display_order: 10,
        });
        
        variables.insert("spark_version".to_string(), TemplateVariable {
            name: "Spark Version".to_string(),
            var_type: VariableType::String,
            default_value: Some(serde_json::json!("3.5.1")),
            description: "Spark version".to_string(),
            required: true,
            group: "scala".to_string(),
            display_order: 11,
        });
        
        // Add remaining groups similar to Python config...
        
        Ok(TemplateConfiguration {
            variables,
            variable_groups: groups,
        })
    }
}



use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub template_type: TemplateType,
    pub version: String,
    pub description: Option<String>,
    pub config: FirestreamConfig,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateType {
    Python,
    NodeJs,
    PySpark,
    #[serde(rename = "pyspark-scala")]
    PySparkScala,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirestreamConfig {
    pub app: AppConfig,
    pub resources: ResourceConfig,
    pub kafka: Option<KafkaConfig>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Option<serde_json::Value>,
    #[serde(rename = "outputStructure")]
    pub output_structure: Option<OutputStructure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub app_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu: String,
    pub memory: String,
    pub gpu: Option<bool>,
    #[serde(rename = "gpuType")]
    pub gpu_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStructure {
    pub files: Vec<String>,
    pub directories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFiles {
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
    pub size: u64,
}

// Template configuration structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfiguration {
    pub variables: HashMap<String, TemplateVariable>,
    pub variable_groups: Vec<VariableGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub var_type: VariableType,
    pub default_value: Option<serde_json::Value>,
    pub description: String,
    pub required: bool,
    pub group: String,
    pub display_order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableType {
    String,
    Boolean,
    Integer,
    Float,
    Array(Box<VariableType>),
    Object,
    Choice(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableGroup {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub collapsed: bool,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateGenerationRequest {
    pub template_id: String,
    pub template_type: String,
    pub name: String,
    pub output_path: String,
    pub variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateGenerationResult {
    pub success: bool,
    pub generated_files: Vec<GeneratedFile>,
    pub output_path: String,
    pub errors: Vec<String>,
}

//! Configuration types for Spark application generation
//!
//! This module defines the configuration structures for generating
//! Spark applications in both Python and Scala.

use crate::config::TemplateConfig;
use crate::error::{Result, TemplatizerError};
use serde::{Deserialize, Serialize};

/// Language/template type for Spark application generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageType {
    /// Python (PySpark) application
    #[default]
    Python,
    /// Scala Spark application
    Scala,
}

impl LanguageType {
    /// Get the language type as string
    pub fn as_str(&self) -> &'static str {
        match self {
            LanguageType::Python => "python",
            LanguageType::Scala => "scala",
        }
    }

    /// Get the file extension for the language
    pub fn extension(&self) -> &'static str {
        match self {
            LanguageType::Python => "py",
            LanguageType::Scala => "scala",
        }
    }

    /// Get the template subdirectory for the language
    pub fn template_dir(&self) -> &'static str {
        match self {
            LanguageType::Python => "python",
            LanguageType::Scala => "scala",
        }
    }
}

impl std::str::FromStr for LanguageType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "python" | "py" | "pyspark" => Ok(LanguageType::Python),
            "scala" => Ok(LanguageType::Scala),
            _ => Err(format!(
                "Invalid language type: {}. Use 'python' or 'scala'",
                s
            )),
        }
    }
}

impl std::fmt::Display for LanguageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Configuration for Spark application generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkAppConfig {
    /// Application name
    pub app_name: String,
    /// Application version
    #[serde(default = "default_version")]
    pub version: String,
    /// Organization (e.g., com.example)
    #[serde(default)]
    pub organization: String,
    /// Language type (Python or Scala)
    #[serde(default)]
    pub language: LanguageType,

    // Docker configuration
    /// Docker registry URL
    #[serde(default)]
    pub docker_registry: String,
    /// Docker image name (defaults to app_name)
    #[serde(default)]
    pub docker_image: String,

    // Kubernetes configuration
    /// Kubernetes namespace
    #[serde(default = "default_namespace")]
    pub namespace: String,
    /// Service account name
    #[serde(default)]
    pub service_account: String,

    // Storage configuration
    /// Enable S3 support
    #[serde(default)]
    pub s3_enabled: bool,
    /// S3 bucket name
    #[serde(default)]
    pub s3_bucket: String,
    /// S3 region
    #[serde(default = "default_s3_region")]
    pub s3_region: String,

    // Delta Lake configuration
    /// Enable Delta Lake support
    #[serde(default)]
    pub delta_enabled: bool,
    /// Delta table path
    #[serde(default)]
    pub delta_table_path: String,

    // Kafka configuration
    /// Enable Kafka support
    #[serde(default)]
    pub kafka_enabled: bool,
    /// Kafka bootstrap servers
    #[serde(default)]
    pub kafka_bootstrap_servers: String,
    /// Kafka topic
    #[serde(default)]
    pub kafka_topic: String,

    // Resource configuration
    /// Driver memory (e.g., "1g")
    #[serde(default = "default_driver_memory")]
    pub driver_memory: String,
    /// Driver cores
    #[serde(default = "default_driver_cores")]
    pub driver_cores: u32,
    /// Executor memory (e.g., "2g")
    #[serde(default = "default_executor_memory")]
    pub executor_memory: String,
    /// Executor cores
    #[serde(default = "default_executor_cores")]
    pub executor_cores: u32,
    /// Number of executors
    #[serde(default = "default_num_executors")]
    pub num_executors: u32,

    // Scala-specific configuration
    /// Scala version (for Scala projects)
    #[serde(default = "default_scala_version")]
    pub scala_version: String,
    /// Spark version
    #[serde(default = "default_spark_version")]
    pub spark_version: String,

    // Python-specific configuration
    /// Python version (for Python projects)
    #[serde(default = "default_python_version")]
    pub python_version: String,

    // Additional dependencies
    /// Extra Maven dependencies (Scala)
    #[serde(default)]
    pub maven_dependencies: Vec<String>,
    /// Extra pip packages (Python)
    #[serde(default)]
    pub pip_packages: Vec<String>,

    // Additional configuration
    /// Additional Spark configuration properties
    #[serde(default)]
    pub spark_conf: std::collections::HashMap<String, String>,
    /// Maintainer email
    #[serde(default)]
    pub maintainer_email: String,
}

// Default value functions
fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_namespace() -> String {
    "default".to_string()
}

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

fn default_driver_memory() -> String {
    "1g".to_string()
}

fn default_driver_cores() -> u32 {
    1
}

fn default_executor_memory() -> String {
    "2g".to_string()
}

fn default_executor_cores() -> u32 {
    1
}

fn default_num_executors() -> u32 {
    2
}

fn default_scala_version() -> String {
    "2.12.18".to_string()
}

fn default_spark_version() -> String {
    "3.5.0".to_string()
}

fn default_python_version() -> String {
    "3.11".to_string()
}

impl Default for SparkAppConfig {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            version: default_version(),
            organization: String::new(),
            language: LanguageType::default(),
            docker_registry: String::new(),
            docker_image: String::new(),
            namespace: default_namespace(),
            service_account: String::new(),
            s3_enabled: false,
            s3_bucket: String::new(),
            s3_region: default_s3_region(),
            delta_enabled: false,
            delta_table_path: String::new(),
            kafka_enabled: false,
            kafka_bootstrap_servers: String::new(),
            kafka_topic: String::new(),
            driver_memory: default_driver_memory(),
            driver_cores: default_driver_cores(),
            executor_memory: default_executor_memory(),
            executor_cores: default_executor_cores(),
            num_executors: default_num_executors(),
            scala_version: default_scala_version(),
            spark_version: default_spark_version(),
            python_version: default_python_version(),
            maven_dependencies: Vec::new(),
            pip_packages: Vec::new(),
            spark_conf: std::collections::HashMap::new(),
            maintainer_email: String::new(),
        }
    }
}

impl TemplateConfig for SparkAppConfig {
    fn project_name(&self) -> &str {
        &self.app_name
    }

    fn validate(&self) -> Result<()> {
        if self.app_name.is_empty() {
            return Err(TemplatizerError::invalid_config("app_name cannot be empty"));
        }

        // Validate S3 configuration if enabled
        if self.s3_enabled && self.s3_bucket.is_empty() {
            return Err(TemplatizerError::invalid_config(
                "s3_bucket is required when s3_enabled is true",
            ));
        }

        // Validate Kafka configuration if enabled
        if self.kafka_enabled && self.kafka_bootstrap_servers.is_empty() {
            return Err(TemplatizerError::invalid_config(
                "kafka_bootstrap_servers is required when kafka_enabled is true",
            ));
        }

        // Validate resource configuration
        if self.driver_cores == 0 {
            return Err(TemplatizerError::invalid_config(
                "driver_cores must be greater than 0",
            ));
        }

        if self.executor_cores == 0 {
            return Err(TemplatizerError::invalid_config(
                "executor_cores must be greater than 0",
            ));
        }

        Ok(())
    }
}

/// Builder for SparkAppConfig
#[derive(Debug, Clone, Default)]
pub struct SparkAppConfigBuilder {
    config: SparkAppConfig,
}

impl SparkAppConfigBuilder {
    /// Create a new builder
    pub fn new(app_name: &str, version: &str) -> Self {
        let mut config = SparkAppConfig::default();
        config.app_name = app_name.to_string();
        config.version = version.to_string();
        Self { config }
    }

    /// Set the language type
    pub fn language(mut self, language: LanguageType) -> Self {
        self.config.language = language;
        self
    }

    /// Set the organization
    pub fn organization(mut self, org: &str) -> Self {
        self.config.organization = org.to_string();
        self
    }

    /// Set the Docker registry
    pub fn docker_registry(mut self, registry: &str) -> Self {
        self.config.docker_registry = registry.to_string();
        self
    }

    /// Set the Kubernetes namespace
    pub fn namespace(mut self, ns: &str) -> Self {
        self.config.namespace = ns.to_string();
        self
    }

    /// Enable S3 support
    pub fn enable_s3(mut self) -> Self {
        self.config.s3_enabled = true;
        self
    }

    /// Set S3 configuration
    pub fn s3_config(mut self, bucket: &str, region: &str) -> Self {
        self.config.s3_enabled = true;
        self.config.s3_bucket = bucket.to_string();
        self.config.s3_region = region.to_string();
        self
    }

    /// Enable Delta Lake support
    pub fn enable_delta(mut self) -> Self {
        self.config.delta_enabled = true;
        self
    }

    /// Set Delta Lake configuration
    pub fn delta_config(mut self, table_path: &str) -> Self {
        self.config.delta_enabled = true;
        self.config.delta_table_path = table_path.to_string();
        self
    }

    /// Enable Kafka support
    pub fn enable_kafka(mut self) -> Self {
        self.config.kafka_enabled = true;
        self
    }

    /// Set Kafka configuration
    pub fn kafka_config(mut self, bootstrap_servers: &str, topic: &str) -> Self {
        self.config.kafka_enabled = true;
        self.config.kafka_bootstrap_servers = bootstrap_servers.to_string();
        self.config.kafka_topic = topic.to_string();
        self
    }

    /// Set driver resources
    pub fn driver_resources(mut self, memory: &str, cores: u32) -> Self {
        self.config.driver_memory = memory.to_string();
        self.config.driver_cores = cores;
        self
    }

    /// Set executor resources
    pub fn executor_resources(mut self, memory: &str, cores: u32, num: u32) -> Self {
        self.config.executor_memory = memory.to_string();
        self.config.executor_cores = cores;
        self.config.num_executors = num;
        self
    }

    /// Add a Maven dependency (Scala)
    pub fn add_maven_dependency(mut self, dependency: &str) -> Self {
        self.config.maven_dependencies.push(dependency.to_string());
        self
    }

    /// Add a pip package (Python)
    pub fn add_pip_package(mut self, package: &str) -> Self {
        self.config.pip_packages.push(package.to_string());
        self
    }

    /// Add a Spark configuration property
    pub fn add_spark_conf(mut self, key: &str, value: &str) -> Self {
        self.config.spark_conf.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<SparkAppConfig> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// Build without validation (for testing)
    pub fn build_unchecked(self) -> SparkAppConfig {
        self.config
    }
}

impl crate::config::ConfigBuilder for SparkAppConfigBuilder {
    type Config = SparkAppConfig;

    fn new() -> Self {
        Self::default()
    }

    fn project_name(mut self, name: impl Into<String>) -> Self {
        self.config.app_name = name.into();
        self
    }

    fn build(self) -> Result<Self::Config> {
        SparkAppConfigBuilder::build(self)
    }
}

/// Get example Python Spark application configuration
pub fn get_python_example() -> SparkAppConfig {
    SparkAppConfigBuilder::new("example-pyspark-app", "1.0.0")
        .language(LanguageType::Python)
        .organization("com.example")
        .docker_registry("docker.io/myorg")
        .namespace("spark-apps")
        .s3_config("my-data-lake", "us-east-1")
        .enable_delta()
        .kafka_config("kafka:9092", "input-topic")
        .driver_resources("1g", 1)
        .executor_resources("2g", 2, 3)
        .add_pip_package("pandas")
        .add_pip_package("pyarrow")
        .add_spark_conf("spark.sql.shuffle.partitions", "200")
        .build_unchecked()
}

/// Get example Scala Spark application configuration
pub fn get_scala_example() -> SparkAppConfig {
    SparkAppConfigBuilder::new("example-scala-app", "1.0.0")
        .language(LanguageType::Scala)
        .organization("com.example")
        .docker_registry("docker.io/myorg")
        .namespace("spark-apps")
        .s3_config("my-data-lake", "us-east-1")
        .enable_delta()
        .kafka_config("kafka:9092", "input-topic")
        .driver_resources("2g", 2)
        .executor_resources("4g", 4, 5)
        .add_maven_dependency("io.delta:delta-core_2.12:2.4.0")
        .add_maven_dependency("org.apache.spark:spark-sql-kafka-0-10_2.12:3.5.0")
        .add_spark_conf("spark.sql.shuffle.partitions", "200")
        .build_unchecked()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_type_from_str() {
        assert!(matches!(
            "python".parse::<LanguageType>(),
            Ok(LanguageType::Python)
        ));
        assert!(matches!(
            "scala".parse::<LanguageType>(),
            Ok(LanguageType::Scala)
        ));
        assert!(matches!(
            "py".parse::<LanguageType>(),
            Ok(LanguageType::Python)
        ));
        assert!(matches!(
            "pyspark".parse::<LanguageType>(),
            Ok(LanguageType::Python)
        ));
        assert!("java".parse::<LanguageType>().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = SparkAppConfigBuilder::new("Test App", "1.0.0")
            .language(LanguageType::Python)
            .organization("com.example")
            .docker_registry("example.io")
            .enable_s3()
            .s3_config("test-bucket", "us-west-2")
            .build()
            .unwrap();

        assert_eq!(config.app_name, "Test App");
        assert_eq!(config.language, LanguageType::Python);
        assert!(config.s3_enabled);
        assert_eq!(config.s3_bucket, "test-bucket");
    }

    #[test]
    fn test_validation_fails_without_required_fields() {
        let config = SparkAppConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_s3_validation() {
        let config = SparkAppConfigBuilder::new("test", "1.0.0")
            .enable_s3()
            .build_unchecked();

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_get_python_example() {
        let config = get_python_example();
        assert!(!config.app_name.is_empty());
        assert_eq!(config.language, LanguageType::Python);
    }

    #[test]
    fn test_get_scala_example() {
        let config = get_scala_example();
        assert!(!config.app_name.is_empty());
        assert_eq!(config.language, LanguageType::Scala);
    }
}

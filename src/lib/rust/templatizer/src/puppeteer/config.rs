//! Configuration types for Puppeteer web scraper generation
//!
//! This module defines the configuration structures for generating
//! Puppeteer-based web scrapers.

use crate::config::TemplateConfig;
use crate::error::{Result, TemplatizerError};
use serde::{Deserialize, Serialize};

/// Workflow type for web scraping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowType {
    /// DOM-based scraping using CSS selectors
    #[default]
    DomScraping,
    /// API-based scraping using intercepted network requests
    ApiScraping,
}

impl WorkflowType {
    /// Get the workflow type as string
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowType::DomScraping => "dom-scraping",
            WorkflowType::ApiScraping => "api-scraping",
        }
    }
}

impl std::str::FromStr for WorkflowType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dom" | "dom-scraping" | "dom_scraping" => Ok(WorkflowType::DomScraping),
            "api" | "api-scraping" | "api_scraping" => Ok(WorkflowType::ApiScraping),
            _ => Err(format!(
                "Invalid workflow type: {}. Use 'dom-scraping' or 'api-scraping'",
                s
            )),
        }
    }
}

impl std::fmt::Display for WorkflowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Authentication type for the scraper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthType {
    /// Form-based authentication (username/password)
    #[default]
    Form,
    /// API key authentication
    ApiKey,
    /// OAuth authentication
    OAuth,
    /// Cookie-based authentication
    Cookie,
    /// No authentication required
    None,
}

impl std::str::FromStr for AuthType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "form" => Ok(AuthType::Form),
            "api-key" | "api_key" | "apikey" => Ok(AuthType::ApiKey),
            "oauth" => Ok(AuthType::OAuth),
            "cookie" => Ok(AuthType::Cookie),
            "none" => Ok(AuthType::None),
            _ => Err(format!("Invalid auth type: {}", s)),
        }
    }
}

/// Data field definition for the schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    /// Field name
    pub name: String,
    /// Field type (string, number, boolean, Date, array, object)
    #[serde(rename = "type")]
    pub field_type: String,
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,
    /// Optional description
    #[serde(default)]
    pub description: String,
}

/// DOM extraction rule for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomExtractionRule {
    /// Extraction type (text, attribute, html)
    #[serde(rename = "type")]
    pub extraction_type: String,
    /// CSS selector
    pub selector: String,
    /// Attribute name (for attribute extraction)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
    /// Transform expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
}

/// Navigation step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationStep {
    /// Step name
    pub name: String,
    /// URL path to navigate to
    pub url: String,
    /// Wait type (selector, network, timeout)
    #[serde(default = "default_wait_type")]
    pub wait_type: String,
    /// Wait value (selector or milliseconds)
    pub wait_value: String,
    /// Whether to take a screenshot after navigation
    #[serde(default)]
    pub screenshot: bool,
}

fn default_wait_type() -> String {
    "selector".to_string()
}

/// API endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    /// Endpoint name
    pub name: String,
    /// HTTP method
    #[serde(default = "default_method")]
    pub method: String,
    /// URL path
    pub path: String,
    /// Query parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Request body data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

fn default_method() -> String {
    "GET".to_string()
}

/// Data schema configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataSchema {
    /// Field definitions
    #[serde(default)]
    pub fields: Vec<DataField>,
}

/// DOM extraction configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomExtraction {
    /// CSS selector for item container
    #[serde(default)]
    pub item_selector: String,
    /// Field extraction rules
    #[serde(default)]
    pub fields: std::collections::HashMap<String, DomExtractionRule>,
}

/// API extraction configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiExtraction {
    /// Response mappings by endpoint name
    #[serde(default)]
    pub response_mappings: std::collections::HashMap<String, ResponseMapping>,
}

/// Response mapping for API extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMapping {
    /// JSON path to data
    pub data_path: String,
    /// Transform expression for items
    pub item_transform: String,
    /// Optional filter expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Field name to validate
    pub field: String,
    /// Rule type (required, min, max, pattern)
    pub rule: String,
    /// Validation value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    /// Error message
    #[serde(default)]
    pub message: String,
}

/// Configuration for Puppeteer scraper generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperConfig {
    /// Site name (used as project name)
    pub site_name: String,
    /// Target site URL
    pub site_url: String,
    /// Workflow type (DOM or API scraping)
    #[serde(default)]
    pub workflow_type: WorkflowType,

    // Authentication
    /// Authentication type
    #[serde(default)]
    pub auth_type: AuthType,
    /// Login page path
    #[serde(default)]
    pub login_path: String,
    /// Username input selector
    #[serde(default = "default_username_selector")]
    pub login_username_selector: String,
    /// Password input selector
    #[serde(default = "default_password_selector")]
    pub login_password_selector: String,
    /// Submit button selector
    #[serde(default = "default_submit_selector")]
    pub login_submit_selector: String,
    /// Success indicator selector
    #[serde(default = "default_success_selector")]
    pub login_success_selector: String,
    /// Error indicator selector
    #[serde(default)]
    pub login_error_selector: String,

    // Storage configuration
    /// S3 bucket name
    #[serde(default)]
    pub s3_bucket: String,
    /// S3 region
    #[serde(default = "default_s3_region")]
    pub s3_region: String,
    /// Output format (parquet, json, csv)
    #[serde(default = "default_output_format")]
    pub output_format: String,

    // Retry configuration
    /// Enable retry on failure
    #[serde(default = "default_true")]
    pub enable_retry: bool,
    /// Number of retry attempts
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,
    /// Retry backoff in milliseconds
    #[serde(default = "default_retry_backoff")]
    pub retry_backoff: u32,

    // Navigation steps (for DOM scraping)
    #[serde(default)]
    pub navigation_steps: Vec<NavigationStep>,

    // Data schema
    #[serde(default)]
    pub data_schema: DataSchema,

    // DOM extraction rules
    #[serde(default)]
    pub dom_extraction: DomExtraction,

    // API endpoints (for API scraping)
    #[serde(default)]
    pub api_endpoints: Vec<ApiEndpoint>,

    // API extraction configuration
    #[serde(default)]
    pub api_extraction: ApiExtraction,

    // API authentication storage
    #[serde(default = "default_auth_storage")]
    pub auth_storage: String,
    #[serde(default = "default_auth_token_key")]
    pub auth_token_key: String,
    #[serde(default)]
    pub auth_header_format: String,

    // Validation rules
    #[serde(default)]
    pub validation_rules: Vec<ValidationRule>,

    // Custom transformations
    #[serde(default)]
    pub transformations: std::collections::HashMap<String, String>,

    // Docker configuration
    #[serde(default)]
    pub docker_registry: String,
    #[serde(default)]
    pub maintainer_email: String,
}

// Default value functions
fn default_username_selector() -> String {
    "#username".to_string()
}

fn default_password_selector() -> String {
    "#password".to_string()
}

fn default_submit_selector() -> String {
    "button[type='submit']".to_string()
}

fn default_success_selector() -> String {
    ".dashboard".to_string()
}

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

fn default_output_format() -> String {
    "parquet".to_string()
}

fn default_true() -> bool {
    true
}

fn default_retry_attempts() -> u32 {
    3
}

fn default_retry_backoff() -> u32 {
    1000
}

fn default_auth_storage() -> String {
    "localStorage".to_string()
}

fn default_auth_token_key() -> String {
    "auth_token".to_string()
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            site_name: String::new(),
            site_url: String::new(),
            workflow_type: WorkflowType::default(),
            auth_type: AuthType::default(),
            login_path: String::new(),
            login_username_selector: default_username_selector(),
            login_password_selector: default_password_selector(),
            login_submit_selector: default_submit_selector(),
            login_success_selector: default_success_selector(),
            login_error_selector: String::new(),
            s3_bucket: String::new(),
            s3_region: default_s3_region(),
            output_format: default_output_format(),
            enable_retry: true,
            retry_attempts: default_retry_attempts(),
            retry_backoff: default_retry_backoff(),
            navigation_steps: Vec::new(),
            data_schema: DataSchema::default(),
            dom_extraction: DomExtraction::default(),
            api_endpoints: Vec::new(),
            api_extraction: ApiExtraction::default(),
            auth_storage: default_auth_storage(),
            auth_token_key: default_auth_token_key(),
            auth_header_format: String::new(),
            validation_rules: Vec::new(),
            transformations: std::collections::HashMap::new(),
            docker_registry: String::new(),
            maintainer_email: String::new(),
        }
    }
}

impl TemplateConfig for ScraperConfig {
    fn project_name(&self) -> &str {
        &self.site_name
    }

    fn validate(&self) -> Result<()> {
        if self.site_name.is_empty() {
            return Err(TemplatizerError::invalid_config("site_name cannot be empty"));
        }

        if self.site_url.is_empty() {
            return Err(TemplatizerError::invalid_config("site_url cannot be empty"));
        }

        // Validate data schema has fields
        if self.data_schema.fields.is_empty() {
            return Err(TemplatizerError::invalid_config(
                "data_schema.fields cannot be empty",
            ));
        }

        // Workflow-specific validation
        match self.workflow_type {
            WorkflowType::DomScraping => {
                if self.dom_extraction.item_selector.is_empty() {
                    return Err(TemplatizerError::invalid_config(
                        "dom_extraction.item_selector is required for DOM scraping",
                    ));
                }

                // Check that all schema fields have extraction rules
                for field in &self.data_schema.fields {
                    if !self.dom_extraction.fields.contains_key(&field.name) {
                        return Err(TemplatizerError::invalid_config(format!(
                            "No extraction rule for field '{}'",
                            field.name
                        )));
                    }
                }
            }
            WorkflowType::ApiScraping => {
                if self.api_endpoints.is_empty() {
                    return Err(TemplatizerError::invalid_config(
                        "api_endpoints cannot be empty for API scraping",
                    ));
                }

                // Check that all endpoints have response mappings
                for endpoint in &self.api_endpoints {
                    if !self.api_extraction.response_mappings.contains_key(&endpoint.name) {
                        return Err(TemplatizerError::invalid_config(format!(
                            "No response mapping for endpoint '{}'",
                            endpoint.name
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Builder for ScraperConfig
#[derive(Debug, Clone, Default)]
pub struct ScraperConfigBuilder {
    config: ScraperConfig,
}

impl ScraperConfigBuilder {
    /// Create a new builder
    pub fn new(site_name: &str, site_url: &str) -> Self {
        let mut config = ScraperConfig::default();
        config.site_name = site_name.to_string();
        config.site_url = site_url.to_string();
        Self { config }
    }

    /// Set the workflow type
    pub fn workflow_type(mut self, workflow_type: WorkflowType) -> Self {
        self.config.workflow_type = workflow_type;
        self
    }

    /// Set the authentication type
    pub fn auth_type(mut self, auth_type: AuthType) -> Self {
        self.config.auth_type = auth_type;
        self
    }

    /// Set the S3 bucket
    pub fn s3_bucket(mut self, bucket: &str) -> Self {
        self.config.s3_bucket = bucket.to_string();
        self
    }

    /// Set the S3 region
    pub fn s3_region(mut self, region: &str) -> Self {
        self.config.s3_region = region.to_string();
        self
    }

    /// Set the Docker registry
    pub fn docker_registry(mut self, registry: &str) -> Self {
        self.config.docker_registry = registry.to_string();
        self
    }

    /// Set login selectors for form auth
    pub fn login_selectors(
        mut self,
        username: &str,
        password: &str,
        submit: &str,
        success: &str,
        error: &str,
    ) -> Self {
        self.config.login_username_selector = username.to_string();
        self.config.login_password_selector = password.to_string();
        self.config.login_submit_selector = submit.to_string();
        self.config.login_success_selector = success.to_string();
        self.config.login_error_selector = error.to_string();
        self
    }

    /// Add a data schema field
    pub fn add_data_field(mut self, name: &str, field_type: &str, required: bool) -> Self {
        self.config.data_schema.fields.push(DataField {
            name: name.to_string(),
            field_type: field_type.to_string(),
            required,
            description: String::new(),
        });
        self
    }

    /// Add a DOM extraction rule
    pub fn add_dom_extraction(
        mut self,
        field_name: &str,
        extraction_type: &str,
        selector: &str,
        attribute: Option<&str>,
        transform: Option<&str>,
    ) -> Self {
        self.config.dom_extraction.fields.insert(
            field_name.to_string(),
            DomExtractionRule {
                extraction_type: extraction_type.to_string(),
                selector: selector.to_string(),
                attribute: attribute.map(|s| s.to_string()),
                transform: transform.map(|s| s.to_string()),
            },
        );
        self
    }

    /// Set the DOM item selector
    pub fn item_selector(mut self, selector: &str) -> Self {
        self.config.dom_extraction.item_selector = selector.to_string();
        self
    }

    /// Add a navigation step
    pub fn add_navigation_step(
        mut self,
        name: &str,
        url: &str,
        wait_type: &str,
        wait_value: &str,
        screenshot: bool,
    ) -> Self {
        self.config.navigation_steps.push(NavigationStep {
            name: name.to_string(),
            url: url.to_string(),
            wait_type: wait_type.to_string(),
            wait_value: wait_value.to_string(),
            screenshot,
        });
        self
    }

    /// Add an API endpoint
    pub fn add_api_endpoint(
        mut self,
        name: &str,
        method: &str,
        path: &str,
        params: Option<serde_json::Value>,
        data: Option<serde_json::Value>,
    ) -> Self {
        self.config.api_endpoints.push(ApiEndpoint {
            name: name.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            params,
            data,
        });
        self
    }

    /// Add an API response mapping
    pub fn add_response_mapping(
        mut self,
        endpoint_name: &str,
        data_path: &str,
        item_transform: &str,
        filter: Option<&str>,
    ) -> Self {
        self.config.api_extraction.response_mappings.insert(
            endpoint_name.to_string(),
            ResponseMapping {
                data_path: data_path.to_string(),
                item_transform: item_transform.to_string(),
                filter: filter.map(|s| s.to_string()),
            },
        );
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<ScraperConfig> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// Build without validation (for testing)
    pub fn build_unchecked(self) -> ScraperConfig {
        self.config
    }
}

impl crate::config::ConfigBuilder for ScraperConfigBuilder {
    type Config = ScraperConfig;

    fn new() -> Self {
        Self::default()
    }

    fn project_name(mut self, name: impl Into<String>) -> Self {
        self.config.site_name = name.into();
        self
    }

    fn build(self) -> Result<Self::Config> {
        ScraperConfigBuilder::build(self)
    }
}

/// Get example DOM scraper configuration
pub fn get_dom_example() -> ScraperConfig {
    ScraperConfigBuilder::new("example_shop", "https://shop.example.com")
        .workflow_type(WorkflowType::DomScraping)
        .auth_type(AuthType::Form)
        .s3_bucket("my-data-lake")
        .docker_registry("docker.io/myorg")
        .login_selectors(
            "input[name='email']",
            "input[name='password']",
            "button[type='submit']",
            ".account-dashboard",
            ".error-message",
        )
        .item_selector(".product-item")
        .add_data_field("id", "string", true)
        .add_data_field("name", "string", true)
        .add_data_field("price", "number", false)
        .add_data_field("inStock", "boolean", false)
        .add_dom_extraction("id", "attribute", "[data-product-id]", Some("data-product-id"), None)
        .add_dom_extraction("name", "text", ".product-title", None, None)
        .add_dom_extraction(
            "price",
            "text",
            ".price",
            None,
            Some("text => parseFloat(text.replace(/[^0-9.]/g, ''))"),
        )
        .add_dom_extraction(
            "inStock",
            "text",
            ".availability",
            None,
            Some("text => text.toLowerCase().includes('in stock')"),
        )
        .add_navigation_step("products", "/products", "selector", ".product-grid", true)
        .build_unchecked()
}

/// Get example API scraper configuration
pub fn get_api_example() -> ScraperConfig {
    ScraperConfigBuilder::new("example_api", "https://api.example.com")
        .workflow_type(WorkflowType::ApiScraping)
        .auth_type(AuthType::Form)
        .s3_bucket("my-data-lake")
        .docker_registry("docker.io/myorg")
        .add_data_field("id", "string", true)
        .add_data_field("name", "string", true)
        .add_data_field("quantity", "number", false)
        .add_api_endpoint(
            "products",
            "GET",
            "/api/v1/products",
            Some(serde_json::json!({"page": 1, "limit": 100})),
            None,
        )
        .add_api_endpoint(
            "inventory",
            "POST",
            "/api/v1/inventory",
            None,
            Some(serde_json::json!({"status": "available"})),
        )
        .add_response_mapping(
            "products",
            "data.items",
            "item => ({ id: item.sku, name: item.title, quantity: 0 })",
            None,
        )
        .add_response_mapping(
            "inventory",
            "results",
            "item => ({ id: item.productId, name: item.productName, quantity: item.available })",
            Some("item => item.available > 0"),
        )
        .build_unchecked()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_type_from_str() {
        assert!(matches!(
            "dom-scraping".parse::<WorkflowType>(),
            Ok(WorkflowType::DomScraping)
        ));
        assert!(matches!(
            "api-scraping".parse::<WorkflowType>(),
            Ok(WorkflowType::ApiScraping)
        ));
        assert!(matches!(
            "dom".parse::<WorkflowType>(),
            Ok(WorkflowType::DomScraping)
        ));
        assert!(matches!(
            "api".parse::<WorkflowType>(),
            Ok(WorkflowType::ApiScraping)
        ));
        assert!("invalid".parse::<WorkflowType>().is_err());
    }

    #[test]
    fn test_dom_config_builder() {
        let config = ScraperConfigBuilder::new("test_shop", "https://shop.test.com")
            .workflow_type(WorkflowType::DomScraping)
            .s3_bucket("test-bucket")
            .docker_registry("test.io")
            .item_selector(".product-item")
            .add_data_field("id", "string", true)
            .add_dom_extraction("id", "attribute", "[data-id]", Some("data-id"), None)
            .add_navigation_step("products", "/products", "selector", ".product-list", true)
            .build()
            .unwrap();

        assert_eq!(config.site_name, "test_shop");
        assert_eq!(config.workflow_type, WorkflowType::DomScraping);
        assert_eq!(config.s3_bucket, "test-bucket");
        assert_eq!(config.navigation_steps.len(), 1);
        assert_eq!(config.navigation_steps[0].name, "products");
    }

    #[test]
    fn test_api_config_builder() {
        let config = ScraperConfigBuilder::new("test_api", "https://api.test.com")
            .workflow_type(WorkflowType::ApiScraping)
            .add_data_field("id", "string", true)
            .add_api_endpoint("users", "GET", "/users", None, None)
            .add_response_mapping("users", "data", "item => item", None)
            .build()
            .unwrap();

        assert_eq!(config.workflow_type, WorkflowType::ApiScraping);
        assert_eq!(config.api_endpoints.len(), 1);
    }

    #[test]
    fn test_validation_fails_without_required_fields() {
        let config = ScraperConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_get_dom_example() {
        let config = get_dom_example();
        assert!(!config.site_name.is_empty());
        assert!(!config.dom_extraction.item_selector.is_empty());
    }

    #[test]
    fn test_get_api_example() {
        let config = get_api_example();
        assert!(!config.site_name.is_empty());
        assert!(!config.api_endpoints.is_empty());
    }
}

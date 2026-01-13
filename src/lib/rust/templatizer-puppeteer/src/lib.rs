//! Functional Scraper Generator - Generate production-ready functional web scrapers
//!
//! This library provides two approaches:
//! 1. Legacy templates - Generate scrapers that require manual implementation
//! 2. Functional templates - Generate fully functional scrapers from configuration

pub mod example;
pub mod generator;

use tera::{Tera, Context};
use std::path::Path;
use std::fs;

/// Workflow type for web scraping
#[derive(Debug, Clone, Copy)]
pub enum WorkflowType {
    DomScraping,
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dom" | "dom-scraping" => Ok(WorkflowType::DomScraping),
            "api" | "api-scraping" => Ok(WorkflowType::ApiScraping),
            _ => Err(format!("Invalid workflow type: {}. Use 'dom-scraping' or 'api-scraping'", s)),
        }
    }
}

/// Legacy template generator (for backward compatibility)
pub struct DomScraperTemplatizer {
    tera: Tera,
}

impl DomScraperTemplatizer {
    /// Create a new templatizer
    pub fn new() -> Result<Self, tera::Error> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| ".".to_string());
        let template_path = Path::new(&manifest_dir)
            .join("src/templates/dom_scraper_boilerplate/**/*.tera");

        let tera = Tera::new(template_path.to_str().unwrap())?;
        Ok(Self { tera })
    }

    /// Render all templates to the specified directory
    pub fn render_to_directory(
        &mut self,
        context: Context,
        output_dir: &Path
    ) -> Result<(), Box<dyn std::error::Error>> {
        render_templates_generic(&mut self.tera, context, output_dir, "dom_scraper_boilerplate")
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

/// Functional template generator (new approach)
pub struct FunctionalScraperTemplatizer {
    tera: Tera,
}

impl FunctionalScraperTemplatizer {
    /// Create a new functional templatizer
    pub fn new() -> Result<Self, tera::Error> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| ".".to_string());
        let template_path = Path::new(&manifest_dir)
            .join("src/templates/fn_scraper_boilerplate/**/*.tera");

        let tera = Tera::new(template_path.to_str().unwrap())?;
        Ok(Self { tera })
    }

    /// Render all templates to the specified directory
    pub fn render_to_directory(
        &mut self,
        context: Context,
        output_dir: &Path
    ) -> Result<(), Box<dyn std::error::Error>> {
        render_templates_generic(&mut self.tera, context, output_dir, "fn_scraper_boilerplate")
    }

    /// Render templates with generated code
    pub fn render_with_generated_code(
        &mut self,
        config: serde_json::Value,
        output_dir: &Path
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate configuration
        generator::validate_functional_config(&config)?;

        // Generate TypeScript code
        let scraper_code = generator::generate_scraper_definition(&config)?;

        // Create context with generated code
        let context = Context::from_serialize(&config)?;

        // Instead of inserting into context, write directly to files
        let src_dir = output_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        // Write the generated scraper definition
        let scraper_path = src_dir.join("scraper-definition.ts");
        fs::write(&scraper_path, scraper_code)?;

        // Generate API scraper if needed
        if config["workflow_type"] == "api-scraping" {
            let api_code = generator::generate_api_scraper_definition(&config)?;
            let api_path = src_dir.join("api-scraper-definition.ts");
            fs::write(&api_path, api_code)?;
        }

        // Now render the rest of the templates
        self.render_to_directory(context, output_dir)?;

        Ok(())
    }
}

/// Generic template rendering function
fn render_templates_generic(
    tera: &mut Tera,
    context: Context,
    output_dir: &Path,
    template_prefix: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all(output_dir)?;

    // Render each template
    for template_name in tera.get_template_names() {
        // Skip non-.tera files
        if !template_name.ends_with(".tera") {
            continue;
        }

        // Skip the scraper-definition.ts.tera file for functional templates
        // since we generate it programmatically
        if template_prefix == "fn_scraper_boilerplate" &&
           (template_name.contains("scraper-definition.ts.tera") ||
            template_name.contains("api-scraper-definition.ts.tera")) {
            continue;
        }

        // Render template
        let rendered = tera.render(template_name, &context)?;

        // Determine output path (remove .tera extension)
        let output_file = template_name.trim_end_matches(".tera");

        // Remove the template prefix from the path
        let relative_path = output_file
            .strip_prefix(&format!("{}/", template_prefix))
            .unwrap_or(output_file);

        let output_path = output_dir.join(relative_path);

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write file
        fs::write(&output_path, rendered)?;
    }

    Ok(())
}

/// Configuration builder for web scraper applications
pub struct ScraperConfig {
    config: serde_json::Map<String, serde_json::Value>,
}

impl ScraperConfig {
    /// Create a new configuration builder
    pub fn new(site_name: &str, site_url: &str, workflow_type: WorkflowType) -> Self {
        let mut config = serde_json::Map::new();
        config.insert("site_name".to_string(), serde_json::json!(site_name));
        config.insert("site_url".to_string(), serde_json::json!(site_url));
        config.insert("workflow_type".to_string(), serde_json::json!(workflow_type.as_str()));

        // Default values
        config.insert("auth_type".to_string(), serde_json::json!("form"));
        config.insert("output_format".to_string(), serde_json::json!("parquet"));
        config.insert("s3_region".to_string(), serde_json::json!("us-east-1"));
        config.insert("enable_retry".to_string(), serde_json::json!(true));
        config.insert("retry_attempts".to_string(), serde_json::json!(3));
        config.insert("retry_backoff".to_string(), serde_json::json!(1000));

        Self { config }
    }

    /// Set authentication type
    pub fn auth_type(mut self, auth_type: &str) -> Self {
        self.config.insert("auth_type".to_string(), serde_json::json!(auth_type));
        self
    }

    /// Set S3 bucket
    pub fn s3_bucket(mut self, bucket: &str) -> Self {
        self.config.insert("s3_bucket".to_string(), serde_json::json!(bucket));
        self
    }

    /// Set S3 region
    pub fn s3_region(mut self, region: &str) -> Self {
        self.config.insert("s3_region".to_string(), serde_json::json!(region));
        self
    }

    /// Set Docker registry
    pub fn docker_registry(mut self, registry: &str) -> Self {
        self.config.insert("docker_registry".to_string(), serde_json::json!(registry));
        self
    }

    /// Set login selectors for form auth
    pub fn login_selectors(
        mut self,
        username: &str,
        password: &str,
        submit: &str,
        success: &str,
        error: &str
    ) -> Self {
        self.config.insert("login_username_selector".to_string(), serde_json::json!(username));
        self.config.insert("login_password_selector".to_string(), serde_json::json!(password));
        self.config.insert("login_submit_selector".to_string(), serde_json::json!(submit));
        self.config.insert("login_success_selector".to_string(), serde_json::json!(success));
        self.config.insert("login_error_selector".to_string(), serde_json::json!(error));
        self
    }

    /// Add data schema field (for functional scrapers)
    pub fn add_data_field(
        mut self,
        name: &str,
        field_type: &str,
        required: bool
    ) -> Self {
        let fields = self.config
            .entry("data_schema".to_string())
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .unwrap()
            .entry("fields".to_string())
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap();

        fields.push(serde_json::json!({
            "name": name,
            "type": field_type,
            "required": required
        }));

        self
    }

    /// Add DOM extraction rule (for functional scrapers)
    pub fn add_dom_extraction(
        mut self,
        field_name: &str,
        extraction_type: &str,
        selector: &str,
        options: Option<serde_json::Value>
    ) -> Self {
        let extraction_fields = self.config
            .entry("dom_extraction".to_string())
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .unwrap()
            .entry("fields".to_string())
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .unwrap();

        let mut rule = serde_json::json!({
            "type": extraction_type,
            "selector": selector
        });

        if let Some(opts) = options {
            if let Some(obj) = rule.as_object_mut() {
                if let Some(opts_obj) = opts.as_object() {
                    for (k, v) in opts_obj {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        extraction_fields.insert(field_name.to_string(), rule);

        self
    }

    /// Add navigation step (for DOM scraping)
    pub fn add_navigation_step(
        mut self,
        name: &str,
        url: &str,
        wait_type: &str,
        wait_value: &str,
        screenshot: bool
    ) -> Self {
        let steps = self.config
            .entry("navigation_steps".to_string())
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap();

        steps.push(serde_json::json!({
            "name": name,
            "url": url,
            "wait_type": wait_type,
            "wait_value": wait_value,
            "screenshot": screenshot
        }));

        self
    }

    /// Add API endpoint (for API scraping)
    pub fn add_api_endpoint(
        mut self,
        name: &str,
        method: &str,
        path: &str,
        params: Option<serde_json::Value>,
        data: Option<serde_json::Value>
    ) -> Self {
        let endpoints = self.config
            .entry("api_endpoints".to_string())
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap();

        let mut endpoint = serde_json::json!({
            "name": name,
            "method": method,
            "path": path
        });

        if let Some(p) = params {
            endpoint["params"] = p;
        }
        if let Some(d) = data {
            endpoint["data"] = d;
        }

        endpoints.push(endpoint);

        self
    }

    /// Build the configuration into a Tera context
    pub fn build(self) -> Result<Context, tera::Error> {
        Context::from_serialize(serde_json::Value::Object(self.config))
    }

    /// Build the configuration as JSON value
    pub fn build_json(self) -> serde_json::Value {
        serde_json::Value::Object(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_type_from_str() {
        assert!(matches!("dom-scraping".parse::<WorkflowType>(), Ok(WorkflowType::DomScraping)));
        assert!(matches!("api-scraping".parse::<WorkflowType>(), Ok(WorkflowType::ApiScraping)));
        assert!(matches!("dom".parse::<WorkflowType>(), Ok(WorkflowType::DomScraping)));
        assert!(matches!("api".parse::<WorkflowType>(), Ok(WorkflowType::ApiScraping)));
        assert!("invalid".parse::<WorkflowType>().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = ScraperConfig::new("test_shop", "https://shop.test.com", WorkflowType::DomScraping)
            .s3_bucket("test-bucket")
            .docker_registry("test.io")
            .add_navigation_step("products", "/products", "selector", ".product-list", true)
            .build()
            .unwrap();

        assert_eq!(config.get("site_name").unwrap().as_str().unwrap(), "test_shop");
        assert_eq!(config.get("workflow_type").unwrap().as_str().unwrap(), "dom-scraping");
        assert_eq!(config.get("s3_bucket").unwrap().as_str().unwrap(), "test-bucket");

        let steps = config.get("navigation_steps").unwrap().as_array().unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0]["name"].as_str().unwrap(), "products");
    }

    #[test]
    fn test_functional_config_builder() {
        let config = ScraperConfig::new("test_api", "https://api.test.com", WorkflowType::ApiScraping)
            .add_data_field("id", "string", true)
            .add_data_field("name", "string", true)
            .add_data_field("price", "number", false)
            .build_json();

        let fields = config["data_schema"]["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0]["name"].as_str().unwrap(), "id");
        assert_eq!(fields[0]["type"].as_str().unwrap(), "string");
        assert_eq!(fields[0]["required"].as_bool().unwrap(), true);
    }
}

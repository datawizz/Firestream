//! Code generator for functional scrapers
//! 
//! This module generates complete TypeScript code for scraper definitions
//! based on JSON configuration, eliminating the need for manual implementation.

use serde_json::{Value, json};
use std::fmt::Write;

/// Validate that a functional scraper configuration has all required fields
pub fn validate_functional_config(config: &Value) -> Result<(), String> {
    // Check required top-level fields
    let required_fields = ["site_name", "site_url", "workflow_type", "data_schema"];
    for field in &required_fields {
        if config.get(field).is_none() {
            return Err(format!("Missing required field: {}", field));
        }
    }
    
    // Validate workflow type
    let workflow = config["workflow_type"].as_str()
        .ok_or("workflow_type must be a string")?;
    if workflow != "dom-scraping" && workflow != "api-scraping" {
        return Err("workflow_type must be 'dom-scraping' or 'api-scraping'".to_string());
    }
    
    // Validate data schema
    let data_schema = config.get("data_schema")
        .ok_or("Missing data_schema")?;
    let fields = data_schema.get("fields")
        .and_then(|f| f.as_array())
        .ok_or("data_schema.fields must be an array")?;
    
    if fields.is_empty() {
        return Err("data_schema.fields cannot be empty".to_string());
    }
    
    // Validate each field
    for (i, field) in fields.iter().enumerate() {
        let name = field.get("name")
            .and_then(|n| n.as_str())
            .ok_or(format!("Field {} missing 'name'", i))?;
        let field_type = field.get("type")
            .and_then(|t| t.as_str())
            .ok_or(format!("Field '{}' missing 'type'", name))?;
        
        // Validate type
        let valid_types = ["string", "number", "boolean", "Date", "array", "object"];
        if !valid_types.contains(&field_type) {
            return Err(format!("Field '{}' has invalid type '{}'. Valid types: {:?}", 
                name, field_type, valid_types));
        }
    }
    
    // Validate workflow-specific extraction rules
    match workflow {
        "dom-scraping" => {
            let dom_extraction = config.get("dom_extraction")
                .ok_or("Missing dom_extraction for DOM scraping workflow")?;
            
            dom_extraction.get("item_selector")
                .and_then(|s| s.as_str())
                .ok_or("dom_extraction.item_selector is required")?;
            
            let extraction_fields = dom_extraction.get("fields")
                .ok_or("dom_extraction.fields is required")?;
            
            // Check that all schema fields have extraction rules
            for field in fields {
                let name = field["name"].as_str().unwrap();
                if extraction_fields.get(name).is_none() {
                    return Err(format!("No extraction rule for field '{}'", name));
                }
            }
        }
        "api-scraping" => {
            let api_endpoints = config.get("api_endpoints")
                .and_then(|e| e.as_array())
                .ok_or("api_endpoints is required for API scraping")?;
            
            if api_endpoints.is_empty() {
                return Err("api_endpoints cannot be empty".to_string());
            }
            
            let api_extraction = config.get("api_extraction")
                .ok_or("Missing api_extraction for API scraping workflow")?;
            
            let response_mappings = api_extraction.get("response_mappings")
                .ok_or("api_extraction.response_mappings is required")?;
            
            // Check that all endpoints have mappings
            for endpoint in api_endpoints {
                let name = endpoint["name"].as_str()
                    .ok_or("API endpoint missing 'name'")?;
                if response_mappings.get(name).is_none() {
                    return Err(format!("No response mapping for endpoint '{}'", name));
                }
            }
        }
        _ => {}
    }
    
    Ok(())
}

/// Generate TypeScript code for the scraper definition
pub fn generate_scraper_definition(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    
    // Import statements
    writeln!(&mut code, "import {{")?;
    writeln!(&mut code, "  ScraperDefinition,")?;
    writeln!(&mut code, "  Scraper,")?;
    writeln!(&mut code, "  Extractors,")?;
    writeln!(&mut code, "  CssSelector,")?;
    writeln!(&mut code, "  Url,")?;
    writeln!(&mut code, "  Effect,")?;
    writeln!(&mut code, "  Schema,")?;
    writeln!(&mut code, "  ProcessingError,")?;
    writeln!(&mut code, "  validateData,")?;
    writeln!(&mut code, "  transformToParquet,")?;
    writeln!(&mut code, "  login")?;
    writeln!(&mut code, "}} from '../lib';")?;
    writeln!(&mut code)?;
    
    // Generate configuration schema
    writeln!(&mut code, "// Configuration schema")?;
    writeln!(&mut code, "{}", generate_config_schema(config)?)?;
    writeln!(&mut code)?;
    
    // Generate data schema
    writeln!(&mut code, "// Data schema")?;
    writeln!(&mut code, "{}", generate_data_schema(config)?)?;
    writeln!(&mut code)?;
    
    // Generate scraper definition
    let site_name = config["site_name"].as_str().unwrap_or("site");
    let site_name_pascal = to_pascal_case(site_name);
    
    writeln!(&mut code, "// Scraper definition")?;
    writeln!(&mut code, "export const {}Scraper: ScraperDefinition<Config, {}Data> = {{", 
        site_name, site_name_pascal)?;
    writeln!(&mut code, "  configSchema: ConfigSchema,")?;
    writeln!(&mut code)?;
    
    // Generate login program
    writeln!(&mut code, "  // Login program")?;
    writeln!(&mut code, "{}", generate_login_program(config)?)?;
    writeln!(&mut code)?;
    
    // Generate extraction program
    writeln!(&mut code, "  // Extraction program")?;
    writeln!(&mut code, "{}", generate_extraction_program(config)?)?;
    writeln!(&mut code)?;
    
    // Generate data processing
    writeln!(&mut code, "  // Data processing")?;
    writeln!(&mut code, "{}", generate_process_data(config)?)?;
    writeln!(&mut code)?;
    
    // Generate data transformation
    writeln!(&mut code, "  // Data transformation")?;
    writeln!(&mut code, "{}", generate_transform_data(config)?)?;
    
    writeln!(&mut code, "}};")?;
    
    Ok(code)
}

/// Generate TypeScript code for API scraper definition
pub fn generate_api_scraper_definition(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    
    // Import statements
    writeln!(&mut code, "import {{")?;
    writeln!(&mut code, "  APIScraperDefinition,")?;
    writeln!(&mut code, "  Scraper,")?;
    writeln!(&mut code, "  Extractors,")?;
    writeln!(&mut code, "  CssSelector,")?;
    writeln!(&mut code, "  Url,")?;
    writeln!(&mut code, "  Effect,")?;
    writeln!(&mut code, "  Schema,")?;
    writeln!(&mut code, "  ProcessingError,")?;
    writeln!(&mut code, "  validateData,")?;
    writeln!(&mut code, "  transformToParquet,")?;
    writeln!(&mut code, "  login,")?;
    writeln!(&mut code, "  APIRequest,")?;
    writeln!(&mut code, "  APIResponse")?;
    writeln!(&mut code, "}} from '../lib';")?;
    writeln!(&mut code)?;
    
    // Generate configuration schema
    writeln!(&mut code, "// Configuration schema")?;
    writeln!(&mut code, "{}", generate_api_config_schema(config)?)?;
    writeln!(&mut code)?;
    
    // Generate data schema
    writeln!(&mut code, "// Data schema")?;
    writeln!(&mut code, "{}", generate_data_schema(config)?)?;
    writeln!(&mut code)?;
    
    // Generate API scraper definition
    let site_name = config["site_name"].as_str().unwrap_or("site");
    let site_name_pascal = to_pascal_case(site_name);
    
    writeln!(&mut code, "// API scraper definition")?;
    writeln!(&mut code, "export const {}APIScraper: APIScraperDefinition<Config, {}APIData> = {{", 
        site_name, site_name_pascal)?;
    writeln!(&mut code, "  configSchema: ConfigSchema,")?;
    writeln!(&mut code)?;
    
    // Generate login program
    writeln!(&mut code, "  // Login program to extract auth")?;
    writeln!(&mut code, "{}", generate_api_login_program(config)?)?;
    writeln!(&mut code)?;
    
    // Generate extraction program (minimal for API)
    writeln!(&mut code, "  // Extraction program (not used in API workflow)")?;
    writeln!(&mut code, "  buildExtractionProgram: (_config) =>")?;
    writeln!(&mut code, "    Scraper.extract(Extractors.text(CssSelector('body'))),")?;
    writeln!(&mut code)?;
    
    // Generate API requests
    writeln!(&mut code, "  // API requests")?;
    writeln!(&mut code, "{}", generate_api_requests(config)?)?;
    writeln!(&mut code)?;
    
    // Generate response processing
    writeln!(&mut code, "  // Process API responses")?;
    writeln!(&mut code, "{}", generate_api_response_processing(config)?)?;
    writeln!(&mut code)?;
    
    // Generate data processing
    writeln!(&mut code, "  // Data processing")?;
    writeln!(&mut code, "{}", generate_process_data(config)?)?;
    writeln!(&mut code)?;
    
    // Generate data transformation
    writeln!(&mut code, "  // Data transformation")?;
    writeln!(&mut code, "{}", generate_transform_data(config)?)?;
    
    writeln!(&mut code, "}};")?;
    
    Ok(code)
}

/// Generate configuration schema
fn generate_config_schema(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    
    writeln!(&mut code, "const ConfigSchema = Schema.Struct({{")?;
    writeln!(&mut code, "  loginUrl: Schema.String.pipe(Schema.brand(\"Url\")),")?;
    
    if config["workflow_type"] == "api-scraping" {
        writeln!(&mut code, "  apiBaseUrl: Schema.String.pipe(Schema.brand(\"Url\")),")?;
    }
    
    writeln!(&mut code, "  credentials: Schema.Struct({{")?;
    
    match config["auth_type"].as_str() {
        Some("form") => {
            writeln!(&mut code, "    username: Schema.String,")?;
            writeln!(&mut code, "    password: Schema.String")?;
        }
        Some("api-key") => {
            writeln!(&mut code, "    apiKey: Schema.String")?;
        }
        _ => {
            writeln!(&mut code, "    // TODO: Add auth fields")?;
        }
    }
    
    writeln!(&mut code, "  }}),")?;
    
    // Add navigation URLs for DOM scraping
    if let Some(steps) = config.get("navigation_steps").and_then(|s| s.as_array()) {
        for step in steps {
            if let Some(name) = step["name"].as_str() {
                let field_name = to_camel_case(&name.replace("-", "_"));
                writeln!(&mut code, "  {}Url: Schema.String.pipe(Schema.brand(\"Url\")),", field_name)?;
            }
        }
    }
    
    writeln!(&mut code, "}});")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "type Config = Schema.Schema.Type<typeof ConfigSchema>;")?;
    
    Ok(code)
}

/// Generate data schema from configuration
fn generate_data_schema(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    let site_name = config["site_name"].as_str().unwrap_or("site");
    let schema_name = format!("{}DataSchema", to_pascal_case(site_name));
    
    writeln!(&mut code, "const {} = Schema.Struct({{", schema_name)?;
    
    if let Some(fields) = config["data_schema"]["fields"].as_array() {
        for (i, field) in fields.iter().enumerate() {
            let name = field["name"].as_str().unwrap();
            let field_type = field["type"].as_str().unwrap();
            let required = field["required"].as_bool().unwrap_or(false);
            
            let schema_type = match field_type {
                "string" => "Schema.String",
                "number" => "Schema.Number",
                "boolean" => "Schema.Boolean",
                "Date" => "Schema.Date",
                _ => "Schema.Unknown",
            };
            
            let field_schema = if required {
                schema_type.to_string()
            } else {
                format!("Schema.optional({})", schema_type)
            };
            
            write!(&mut code, "  {}: {}", name, field_schema)?;
            if i < fields.len() - 1 {
                writeln!(&mut code, ",")?;
            } else {
                writeln!(&mut code)?;
            }
        }
    }
    
    writeln!(&mut code, "}});")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "type {}Data = Schema.Schema.Type<typeof {}>;", 
        to_pascal_case(site_name), schema_name)?;
    
    Ok(code)
}

/// Generate login program
fn generate_login_program(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    
    writeln!(&mut code, "  buildLoginProgram: (config) =>")?;
    writeln!(&mut code, "    Scraper.sequence(")?;
    writeln!(&mut code, "      Scraper.navigate(config.loginUrl as Url),")?;
    writeln!(&mut code, "      login(")?;
    
    match config["auth_type"].as_str() {
        Some("form") => {
            writeln!(&mut code, "        config.credentials.username,")?;
            writeln!(&mut code, "        config.credentials.password,")?;
        }
        Some("api-key") => {
            writeln!(&mut code, "        config.credentials.apiKey,")?;
            writeln!(&mut code, "        '', // No password for API key auth")?;
        }
        _ => {
            writeln!(&mut code, "        '', // TODO: Add credentials")?;
            writeln!(&mut code, "        ''")?;
        }
    }
    
    writeln!(&mut code, "        {{")?;
    writeln!(&mut code, "          usernameField: CssSelector('{}'),", 
        config["login_username_selector"].as_str().unwrap_or("#username"))?;
    writeln!(&mut code, "          passwordField: CssSelector('{}'),", 
        config["login_password_selector"].as_str().unwrap_or("#password"))?;
    writeln!(&mut code, "          submitButton: CssSelector('{}')", 
        config["login_submit_selector"].as_str().unwrap_or("button[type='submit']"))?;
    writeln!(&mut code, "        }}")?;
    writeln!(&mut code, "      ),")?;
    writeln!(&mut code, "      Scraper.waitForSelector(CssSelector('{}'))", 
        config["login_success_selector"].as_str().unwrap_or(".dashboard"))?;
    writeln!(&mut code, "    ),")?;
    
    Ok(code)
}

/// Generate extraction program
fn generate_extraction_program(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    
    writeln!(&mut code, "  buildExtractionProgram: (config) => {{")?;
    
    match config["workflow_type"].as_str() {
        Some("dom-scraping") => {
            generate_dom_extraction_program(&mut code, config)?;
        }
        Some("api-scraping") => {
            // For API scraping, we don't extract from DOM
            writeln!(&mut code, "    // API workflow doesn't use DOM extraction")?;
            writeln!(&mut code, "    return Scraper.extract(Extractors.text(CssSelector('body')));")?;
        }
        _ => {}
    }
    
    writeln!(&mut code, "  }},")?;
    
    Ok(code)
}

/// Generate DOM extraction program
fn generate_dom_extraction_program(code: &mut String, config: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let dom_extraction = &config["dom_extraction"];
    let item_selector = dom_extraction["item_selector"].as_str()
        .unwrap_or(".item");
    
    if let Some(steps) = config.get("navigation_steps").and_then(|s| s.as_array()) {
        if !steps.is_empty() {
            writeln!(code, "    const extractionSteps = [")?;
            
            for (i, step) in steps.iter().enumerate() {
                let name = step["name"].as_str().unwrap();
                let wait_value = step["wait_value"].as_str().unwrap_or(".content");
                let screenshot = step["screenshot"].as_bool().unwrap_or(false);
                
                writeln!(code, "      Scraper.sequence(")?;
                writeln!(code, "        Scraper.navigate(config.{}Url as Url),", 
                    to_camel_case(&name.replace("-", "_")))?;
                writeln!(code, "        Scraper.waitForSelector(CssSelector('{}')),", wait_value)?;
                
                if screenshot {
                    writeln!(code, "        Scraper.screenshot('{}'),", name)?;
                }
                
                writeln!(code, "        Scraper.extractAll(")?;
                writeln!(code, "          CssSelector('{}'),", item_selector)?;
                writeln!(code, "{}", generate_composite_extractor(config)?)?;
                writeln!(code, "        )")?;
                write!(code, "      )")?;
                
                if i < steps.len() - 1 {
                    writeln!(code, ",")?;
                } else {
                    writeln!(code)?;
                }
            }
            
            writeln!(code, "    ];")?;
            writeln!(code)?;
            writeln!(code, "    return Scraper.sequence(")?;
            writeln!(code, "      ...extractionSteps,")?;
            writeln!(code, "      Scraper.map(results => results.flat())")?;
            writeln!(code, "    );")?;
        }
    } else {
        // Single page extraction
        writeln!(code, "    return Scraper.extractAll(")?;
        writeln!(code, "      CssSelector('{}'),", item_selector)?;
        writeln!(code, "{}", generate_composite_extractor(config)?)?;
        writeln!(code, "    );")?;
    }
    
    Ok(())
}

/// Generate composite extractor for data fields
fn generate_composite_extractor(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    let site_name = config["site_name"].as_str().unwrap_or("site");
    let type_name = format!("{}Data", to_pascal_case(site_name));
    
    writeln!(&mut code, "          Extractors.composite<{}>({{", type_name)?;
    
    if let Some(fields) = config["dom_extraction"]["fields"].as_object() {
        let field_count = fields.len();
        for (i, (field_name, extraction_rule)) in fields.iter().enumerate() {
            write!(&mut code, "            {}: ", field_name)?;
            
            match extraction_rule["type"].as_str() {
                Some("text") => {
                    write!(&mut code, "Extractors.text(CssSelector('{}'))",
                        extraction_rule["selector"].as_str().unwrap_or(""))?;
                    
                    if let Some(transform) = extraction_rule["transform"].as_str() {
                        write!(&mut code, ".pipe(Scraper.map({}))", transform)?;
                    }
                }
                Some("attribute") => {
                    write!(&mut code, "Extractors.attribute(CssSelector('{}'), '{}')",
                        extraction_rule["selector"].as_str().unwrap_or(""),
                        extraction_rule["attribute"].as_str().unwrap_or(""))?;
                }
                Some("html") => {
                    write!(&mut code, "Extractors.html(CssSelector('{}'))",
                        extraction_rule["selector"].as_str().unwrap_or(""))?;
                }
                _ => {
                    write!(&mut code, "Extractors.text(CssSelector('{}'))",
                        extraction_rule["selector"].as_str().unwrap_or(""))?;
                }
            }
            
            if i < field_count - 1 {
                writeln!(&mut code, ",")?;
            } else {
                writeln!(&mut code)?;
            }
        }
    }
    
    writeln!(&mut code, "          }})")?;
    
    Ok(code)
}

/// Generate process data function
fn generate_process_data(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    let site_name = config["site_name"].as_str().unwrap_or("site");
    let type_name = format!("{}Data", to_pascal_case(site_name));
    let schema_name = format!("{}DataSchema", to_pascal_case(site_name));
    
    writeln!(&mut code, "  processData: (raw: unknown) =>")?;
    writeln!(&mut code, "    Effect.gen(function* (_) {{")?;
    writeln!(&mut code, "      const parsed = yield* _(")?;
    writeln!(&mut code, "        Schema.decodeUnknown(Schema.Array({}))(raw),", schema_name)?;
    writeln!(&mut code, "        Effect.mapError(error => new ProcessingError('Data validation failed', error))")?;
    writeln!(&mut code, "      );")?;
    writeln!(&mut code)?;
    
    // Generate validation rules
    writeln!(&mut code, "      yield* _(validateData<{}>(data => {{", type_name)?;
    writeln!(&mut code, "        const errors: string[] = [];")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "        if (data.length === 0) {{")?;
    writeln!(&mut code, "          errors.push('No data extracted');")?;
    writeln!(&mut code, "        }}")?;
    writeln!(&mut code)?;
    
    // Add custom validation rules if specified
    if let Some(rules) = config.get("validation_rules").and_then(|r| r.as_array()) {
        for rule in rules {
            generate_validation_rule(&mut code, rule)?;
        }
    }
    
    writeln!(&mut code, "        return {{ valid: errors.length === 0, errors }};")?;
    writeln!(&mut code, "      }})(parsed));")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "      return parsed;")?;
    writeln!(&mut code, "    }}),")?;
    
    Ok(code)
}

/// Generate transform data function
fn generate_transform_data(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    let site_name = config["site_name"].as_str().unwrap_or("site");
    let type_name = format!("{}Data", to_pascal_case(site_name));
    
    writeln!(&mut code, "  transformData: transformToParquet('{}', (item: {}) => ({{", 
        site_name, type_name)?;
    writeln!(&mut code, "    ...item,")?;
    writeln!(&mut code, "    scrapedAt: new Date().toISOString(),")?;
    
    // Add any custom transformations
    if let Some(transforms) = config.get("transformations").and_then(|t| t.as_object()) {
        for (field, transform) in transforms {
            writeln!(&mut code, "    {}: {},", field, transform.as_str().unwrap_or(""))?;
        }
    }
    
    writeln!(&mut code, "  }}))")?;
    
    Ok(code)
}

/// Generate a validation rule
fn generate_validation_rule(code: &mut String, rule: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let field = rule["field"].as_str().unwrap_or("");
    let rule_type = rule["rule"].as_str().unwrap_or("");
    let message = rule["message"].as_str().unwrap_or("Validation failed");
    
    match rule_type {
        "required" => {
            writeln!(code, "        data.forEach((item, index) => {{")?;
            writeln!(code, "          if (!item.{}) {{", field)?;
            writeln!(code, "            errors.push(`Item at index ${{index}}: {}`);}}", message)?;
            writeln!(code, "          }}")?;
            writeln!(code, "        }});")?;
        }
        "min" => {
            if let Some(value) = rule["value"].as_f64() {
                writeln!(code, "        data.forEach((item, index) => {{")?;
                writeln!(code, "          if (item.{} < {}) {{", field, value)?;
                writeln!(code, "            errors.push(`Item at index ${{index}}: {}`);}}", message)?;
                writeln!(code, "          }}")?;
                writeln!(code, "        }});")?;
            }
        }
        _ => {}
    }
    
    Ok(())
}

/// Get example DOM scraper configuration
pub fn get_dom_example() -> Value {
    json!({
        "site_name": "example_shop",
        "site_url": "https://shop.example.com",
        "workflow_type": "dom-scraping",
        
        "auth_type": "form",
        "login_path": "account/login",
        "login_username_selector": "input[name='email']",
        "login_password_selector": "input[name='password']",
        "login_submit_selector": "button[type='submit']",
        "login_success_selector": ".account-dashboard",
        "login_error_selector": ".error-message",
        
        "s3_bucket": "my-data-lake",
        "s3_region": "us-east-1",
        "output_format": "parquet",
        
        "navigation_steps": [
            {
                "name": "products",
                "url": "/products",
                "wait_type": "selector",
                "wait_value": ".product-grid",
                "screenshot": true
            }
        ],
        
        "data_schema": {
            "fields": [
                {
                    "name": "id",
                    "type": "string",
                    "required": true,
                    "description": "Product ID"
                },
                {
                    "name": "name",
                    "type": "string",
                    "required": true,
                    "description": "Product name"
                },
                {
                    "name": "price",
                    "type": "number",
                    "required": false,
                    "description": "Product price"
                },
                {
                    "name": "inStock",
                    "type": "boolean",
                    "required": false,
                    "description": "Stock availability"
                }
            ]
        },
        
        "dom_extraction": {
            "item_selector": ".product-item",
            "fields": {
                "id": {
                    "type": "attribute",
                    "selector": "[data-product-id]",
                    "attribute": "data-product-id"
                },
                "name": {
                    "type": "text",
                    "selector": ".product-title"
                },
                "price": {
                    "type": "text",
                    "selector": ".price",
                    "transform": "text => parseFloat(text.replace(/[^0-9.]/g, ''))"
                },
                "inStock": {
                    "type": "text",
                    "selector": ".availability",
                    "transform": "text => text.toLowerCase().includes('in stock')"
                }
            }
        },
        
        "validation_rules": [
            {
                "field": "id",
                "rule": "required",
                "message": "Product ID is required"
            },
            {
                "field": "price",
                "rule": "min",
                "value": 0,
                "message": "Price must be non-negative"
            }
        ],
        
        "docker_registry": "docker.io/myorg",
        "maintainer_email": "data-team@example.com"
    })
}

/// Get example API scraper configuration
pub fn get_api_example() -> Value {
    json!({
        "site_name": "example_api",
        "site_url": "https://api.example.com",
        "workflow_type": "api-scraping",
        
        "auth_type": "form",
        "login_path": "/login",
        "login_username_selector": "#username",
        "login_password_selector": "#password",
        "login_submit_selector": "button[type='submit']",
        "login_success_selector": ".dashboard",
        
        "auth_storage": "localStorage",
        "auth_token_key": "auth_token",
        "auth_header_format": "Bearer {token}",
        
        "s3_bucket": "my-data-lake",
        "s3_region": "us-east-1",
        "output_format": "parquet",
        
        "api_endpoints": [
            {
                "name": "products",
                "method": "GET",
                "path": "/api/v1/products",
                "params": {
                    "page": 1,
                    "limit": 100
                }
            },
            {
                "name": "inventory",
                "method": "POST",
                "path": "/api/v1/inventory",
                "data": {
                    "status": "available"
                }
            }
        ],
        
        "api_headers": [
            {
                "name": "Accept",
                "value": "application/json"
            }
        ],
        
        "data_schema": {
            "fields": [
                {
                    "name": "id",
                    "type": "string",
                    "required": true
                },
                {
                    "name": "name",
                    "type": "string",
                    "required": true
                },
                {
                    "name": "quantity",
                    "type": "number",
                    "required": false
                }
            ]
        },
        
        "api_extraction": {
            "response_mappings": {
                "products": {
                    "data_path": "data.items",
                    "item_transform": "item => ({ id: item.sku, name: item.title, quantity: 0 })"
                },
                "inventory": {
                    "data_path": "results",
                    "item_transform": "item => ({ id: item.productId, name: item.productName, quantity: item.available })",
                    "filter": "item => item.available > 0"
                }
            }
        },
        
        "docker_registry": "docker.io/myorg",
        "maintainer_email": "data-team@example.com"
    })
}

/// Generate API configuration schema
fn generate_api_config_schema(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    
    writeln!(&mut code, "const ConfigSchema = Schema.Struct({{")?;
    writeln!(&mut code, "  loginUrl: Schema.String.pipe(Schema.brand(\"Url\")),")?;
    writeln!(&mut code, "  apiBaseUrl: Schema.String.pipe(Schema.brand(\"Url\")),")?;
    
    writeln!(&mut code, "  credentials: Schema.Struct({{")?;
    
    match config["auth_type"].as_str() {
        Some("form") => {
            writeln!(&mut code, "    username: Schema.String,")?;
            writeln!(&mut code, "    password: Schema.String")?;
        }
        Some("api-key") => {
            writeln!(&mut code, "    apiKey: Schema.String")?;
        }
        _ => {
            writeln!(&mut code, "    // TODO: Add auth fields")?;
        }
    }
    
    writeln!(&mut code, "  }})")?;
    writeln!(&mut code, "}});")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "type Config = Schema.Schema.Type<typeof ConfigSchema>;")?;
    
    Ok(code)
}

/// Generate API login program
fn generate_api_login_program(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    
    writeln!(&mut code, "  buildLoginProgram: (config) =>")?;
    writeln!(&mut code, "    Scraper.sequence(")?;
    writeln!(&mut code, "      Scraper.navigate(config.loginUrl as Url),")?;
    writeln!(&mut code, "      login(")?;
    
    match config["auth_type"].as_str() {
        Some("form") => {
            writeln!(&mut code, "        config.credentials.username,")?;
            writeln!(&mut code, "        config.credentials.password,")?;
        }
        Some("api-key") => {
            writeln!(&mut code, "        config.credentials.apiKey,")?;
            writeln!(&mut code, "        '', // No password for API key auth")?;
        }
        _ => {
            writeln!(&mut code, "        '',")?;
            writeln!(&mut code, "        ''")?;
        }
    }
    
    writeln!(&mut code, "        {{")?;
    writeln!(&mut code, "          usernameField: CssSelector('{}'),", 
        config["login_username_selector"].as_str().unwrap_or("#username"))?;
    writeln!(&mut code, "          passwordField: CssSelector('{}'),", 
        config["login_password_selector"].as_str().unwrap_or("#password"))?;
    writeln!(&mut code, "          submitButton: CssSelector('{}')", 
        config["login_submit_selector"].as_str().unwrap_or("button[type='submit']"))?;
    writeln!(&mut code, "        }}")?;
    writeln!(&mut code, "      ),")?;
    writeln!(&mut code, "      Scraper.waitForSelector(CssSelector('{}')),",
        config["login_success_selector"].as_str().unwrap_or(".dashboard"))?;
    writeln!(&mut code, "      // Extract auth data")?;
    writeln!(&mut code, "      Scraper.extractAuth(")?;
    writeln!(&mut code, "        '{}',", 
        config["auth_storage"].as_str().unwrap_or("localStorage"))?;
    writeln!(&mut code, "        ['{}', 'access_token', 'jwt']", 
        config["auth_token_key"].as_str().unwrap_or("auth_token"))?;
    writeln!(&mut code, "      )")?;
    writeln!(&mut code, "    ),")?;
    
    Ok(code)
}

/// Generate API requests
fn generate_api_requests(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    
    writeln!(&mut code, "  getAPIRequests: (): APIRequest[] => [")?;
    
    if let Some(endpoints) = config["api_endpoints"].as_array() {
        for (i, endpoint) in endpoints.iter().enumerate() {
            writeln!(&mut code, "    {{")?;
            writeln!(&mut code, "      method: '{}',", 
                endpoint["method"].as_str().unwrap_or("GET"))?;
            writeln!(&mut code, "      endpoint: '{}',", 
                endpoint["path"].as_str().unwrap_or(""))?;
            
            if let Some(params) = endpoint.get("params") {
                writeln!(&mut code, "      params: {},", 
                    serde_json::to_string(params)?)?;
            }
            
            if let Some(data) = endpoint.get("data") {
                writeln!(&mut code, "      data: {},", 
                    serde_json::to_string(data)?)?;
            }
            
            writeln!(&mut code, "      name: '{}'",
                endpoint["name"].as_str().unwrap_or(""))?;
            write!(&mut code, "    }}")?;
            
            if i < endpoints.len() - 1 {
                writeln!(&mut code, ",")?;
            } else {
                writeln!(&mut code)?;
            }
        }
    }
    
    writeln!(&mut code, "  ],")?;
    
    Ok(code)
}

/// Generate API response processing
fn generate_api_response_processing(config: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();
    let site_name = config["site_name"].as_str().unwrap_or("site");
    let type_name = format!("{}APIData", to_pascal_case(site_name));
    let schema_name = format!("{}DataSchema", to_pascal_case(site_name));
    
    writeln!(&mut code, "  processAPIResponses: (responses: APIResponse[]) =>")?;
    writeln!(&mut code, "    Effect.gen(function* (_) {{")?;
    writeln!(&mut code, "      const allData: {}[] = [];", type_name)?;
    writeln!(&mut code)?;
    
    writeln!(&mut code, "      for (const response of responses) {{")?;
    writeln!(&mut code, "        if (!response.data) {{")?;
    writeln!(&mut code, "          console.warn(`No data in response for ${{response.name}}`);")?;
    writeln!(&mut code, "          continue;")?;
    writeln!(&mut code, "        }}")?;
    writeln!(&mut code)?;
    
    if let Some(mappings) = config["api_extraction"]["response_mappings"].as_object() {
        writeln!(&mut code, "        switch (response.name) {{")?;
        
        for (endpoint_name, mapping) in mappings {
            writeln!(&mut code, "          case '{}': {{", endpoint_name)?;
            
            let data_path = mapping["data_path"].as_str().unwrap_or("data");
            let transform = mapping["item_transform"].as_str()
                .unwrap_or("item => item");
            
            if data_path.is_empty() {
                writeln!(&mut code, "            const items = Array.isArray(response.data) ? response.data : [response.data];")?;
            } else {
                writeln!(&mut code, "            const items = response.data.{} || [];", data_path)?;
            }
            
            if let Some(filter) = mapping.get("filter").and_then(|f| f.as_str()) {
                writeln!(&mut code, "            const filtered = items.filter({});", filter)?;
                writeln!(&mut code, "            allData.push(...filtered.map({}));", transform)?;
            } else {
                writeln!(&mut code, "            allData.push(...items.map({}));", transform)?;
            }
            
            writeln!(&mut code, "            break;")?;
            writeln!(&mut code, "          }}")?;
        }
        
        writeln!(&mut code, "          default:")?;
        writeln!(&mut code, "            console.warn(`No mapping for endpoint: ${{response.name}}`);")?;
        writeln!(&mut code, "        }}")?;
    }
    
    writeln!(&mut code, "      }}")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "      // Validate all data")?;
    writeln!(&mut code, "      const validated = yield* _(")?;
    writeln!(&mut code, "        Schema.decodeUnknown(Schema.Array({}))(allData),", schema_name)?;
    writeln!(&mut code, "        Effect.mapError(error => new ProcessingError('API data validation failed', error))")?;
    writeln!(&mut code, "      );")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "      return validated;")?;
    writeln!(&mut code, "    }}),")?;
    
    Ok(code)
}

// Utility functions

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

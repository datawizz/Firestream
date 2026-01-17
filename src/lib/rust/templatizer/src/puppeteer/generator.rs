//! Code generator for Puppeteer web scrapers
//!
//! This module generates complete TypeScript code for scraper definitions
//! based on configuration, eliminating the need for manual implementation.

use crate::config::{GenerationOptions, TemplateConfig};
use crate::embedded::TemplateType;
use crate::engine::TemplateEngine;
use crate::error::{Result, TemplatizerError};
use crate::puppeteer::config::{ScraperConfig, WorkflowType};
use std::fmt::Write;
use std::path::Path;
use tera::Context;
use tracing::{debug, info};

/// Generator for Puppeteer scrapers
pub struct PuppeteerGenerator {
    engine: TemplateEngine,
}

impl PuppeteerGenerator {
    /// Create a new Puppeteer generator
    pub fn new() -> Result<Self> {
        let engine = TemplateEngine::new(TemplateType::Puppeteer)?;
        Ok(Self { engine })
    }

    /// Generate a scraper project to a directory
    pub fn generate(&self, config: &ScraperConfig, output_dir: &Path) -> Result<()> {
        self.generate_with_options(config, output_dir, &GenerationOptions::default())
    }

    /// Generate with specific options
    pub fn generate_with_options(
        &self,
        config: &ScraperConfig,
        output_dir: &Path,
        options: &GenerationOptions,
    ) -> Result<()> {
        // Validate config
        config.validate()?;

        info!("Generating Puppeteer scraper: {}", config.site_name);

        if options.dry_run {
            info!("Dry run - would generate to: {}", output_dir.display());
            return Ok(());
        }

        // Create output directory
        std::fs::create_dir_all(output_dir).map_err(|e| {
            TemplatizerError::output_error(output_dir.to_path_buf(), e.to_string())
        })?;

        // Convert config to context
        let context = config.to_context()?;

        // Render templates
        self.render_templates(config, &context, output_dir, options)?;

        // Generate scraper definition code
        self.generate_scraper_definition(config, output_dir)?;

        info!(
            "Successfully generated scraper to {}",
            output_dir.display()
        );
        Ok(())
    }

    /// Render Tera templates to output directory
    fn render_templates(
        &self,
        config: &ScraperConfig,
        context: &Context,
        output_dir: &Path,
        options: &GenerationOptions,
    ) -> Result<()> {
        let template_subdir = match config.workflow_type {
            WorkflowType::DomScraping => "dom_scraper",
            WorkflowType::ApiScraping => "fn_scraper",
        };

        for template_name in self.engine.list_templates() {
            // Skip templates not for this workflow type
            if !template_name.contains(template_subdir) {
                continue;
            }

            // Skip scraper definition templates - we generate those programmatically
            if template_name.contains("scraper-definition.ts.tera") {
                continue;
            }

            // Determine output path
            let output_file = template_name
                .trim_end_matches(".tera")
                .replace(&format!("{}/", template_subdir), "");

            let output_path = output_dir.join(&output_file);

            // Check for existing file
            if output_path.exists() && !options.overwrite {
                debug!("Skipping existing file: {}", output_path.display());
                continue;
            }

            // Render and write
            self.engine.render_to_file(&template_name, context, &output_path)?;

            if options.verbose {
                info!("Generated: {}", output_path.display());
            }
        }

        Ok(())
    }

    /// Generate the TypeScript scraper definition code
    fn generate_scraper_definition(&self, config: &ScraperConfig, output_dir: &Path) -> Result<()> {
        let src_dir = output_dir.join("src");
        std::fs::create_dir_all(&src_dir).map_err(|e| {
            TemplatizerError::output_error(src_dir.clone(), e.to_string())
        })?;

        // Generate main scraper definition
        let scraper_code = generate_scraper_definition_code(config)?;
        let scraper_path = src_dir.join("scraper-definition.ts");
        std::fs::write(&scraper_path, scraper_code).map_err(|e| {
            TemplatizerError::output_error(scraper_path.clone(), e.to_string())
        })?;

        // Generate API scraper if needed
        if config.workflow_type == WorkflowType::ApiScraping {
            let api_code = generate_api_scraper_definition_code(config)?;
            let api_path = src_dir.join("api-scraper-definition.ts");
            std::fs::write(&api_path, api_code).map_err(|e| {
                TemplatizerError::output_error(api_path.clone(), e.to_string())
            })?;
        }

        Ok(())
    }
}

// Code generation functions

/// Generate TypeScript code for the scraper definition
pub fn generate_scraper_definition_code(config: &ScraperConfig) -> Result<String> {
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
    let site_name = &config.site_name;
    let site_name_pascal = to_pascal_case(site_name);

    writeln!(&mut code, "// Scraper definition")?;
    writeln!(
        &mut code,
        "export const {}Scraper: ScraperDefinition<Config, {}Data> = {{",
        site_name, site_name_pascal
    )?;
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
pub fn generate_api_scraper_definition_code(config: &ScraperConfig) -> Result<String> {
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
    let site_name = &config.site_name;
    let site_name_pascal = to_pascal_case(site_name);

    writeln!(&mut code, "// API scraper definition")?;
    writeln!(
        &mut code,
        "export const {}APIScraper: APIScraperDefinition<Config, {}APIData> = {{",
        site_name, site_name_pascal
    )?;
    writeln!(&mut code, "  configSchema: ConfigSchema,")?;
    writeln!(&mut code)?;

    // Generate login program
    writeln!(&mut code, "  // Login program to extract auth")?;
    writeln!(&mut code, "{}", generate_api_login_program(config)?)?;
    writeln!(&mut code)?;

    // Generate extraction program (minimal for API)
    writeln!(&mut code, "  // Extraction program (not used in API workflow)")?;
    writeln!(&mut code, "  buildExtractionProgram: (_config) =>")?;
    writeln!(
        &mut code,
        "    Scraper.extract(Extractors.text(CssSelector('body'))),"
    )?;
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

// Helper generation functions

fn generate_config_schema(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();

    writeln!(&mut code, "const ConfigSchema = Schema.Struct({{")?;
    writeln!(
        &mut code,
        "  loginUrl: Schema.String.pipe(Schema.brand(\"Url\")),"
    )?;

    if config.workflow_type == WorkflowType::ApiScraping {
        writeln!(
            &mut code,
            "  apiBaseUrl: Schema.String.pipe(Schema.brand(\"Url\")),"
        )?;
    }

    writeln!(&mut code, "  credentials: Schema.Struct({{")?;

    match config.auth_type {
        crate::puppeteer::config::AuthType::Form => {
            writeln!(&mut code, "    username: Schema.String,")?;
            writeln!(&mut code, "    password: Schema.String")?;
        }
        crate::puppeteer::config::AuthType::ApiKey => {
            writeln!(&mut code, "    apiKey: Schema.String")?;
        }
        _ => {
            writeln!(&mut code, "    // TODO: Add auth fields")?;
        }
    }

    writeln!(&mut code, "  }}),")?;

    // Add navigation URLs for DOM scraping
    for step in &config.navigation_steps {
        let field_name = to_camel_case(&step.name.replace('-', "_"));
        writeln!(
            &mut code,
            "  {}Url: Schema.String.pipe(Schema.brand(\"Url\")),",
            field_name
        )?;
    }

    writeln!(&mut code, "}});")?;
    writeln!(&mut code)?;
    writeln!(
        &mut code,
        "type Config = Schema.Schema.Type<typeof ConfigSchema>;"
    )?;

    Ok(code)
}

fn generate_data_schema(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();
    let site_name = &config.site_name;
    let schema_name = format!("{}DataSchema", to_pascal_case(site_name));

    writeln!(&mut code, "const {} = Schema.Struct({{", schema_name)?;

    for (i, field) in config.data_schema.fields.iter().enumerate() {
        let schema_type = match field.field_type.as_str() {
            "string" => "Schema.String",
            "number" => "Schema.Number",
            "boolean" => "Schema.Boolean",
            "Date" => "Schema.Date",
            _ => "Schema.Unknown",
        };

        let field_schema = if field.required {
            schema_type.to_string()
        } else {
            format!("Schema.optional({})", schema_type)
        };

        write!(&mut code, "  {}: {}", field.name, field_schema)?;
        if i < config.data_schema.fields.len() - 1 {
            writeln!(&mut code, ",")?;
        } else {
            writeln!(&mut code)?;
        }
    }

    writeln!(&mut code, "}});")?;
    writeln!(&mut code)?;
    writeln!(
        &mut code,
        "type {}Data = Schema.Schema.Type<typeof {}>;",
        to_pascal_case(site_name),
        schema_name
    )?;

    Ok(code)
}

fn generate_login_program(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();

    writeln!(&mut code, "  buildLoginProgram: (config) =>")?;
    writeln!(&mut code, "    Scraper.sequence(")?;
    writeln!(&mut code, "      Scraper.navigate(config.loginUrl as Url),")?;
    writeln!(&mut code, "      login(")?;

    match config.auth_type {
        crate::puppeteer::config::AuthType::Form => {
            writeln!(&mut code, "        config.credentials.username,")?;
            writeln!(&mut code, "        config.credentials.password,")?;
        }
        crate::puppeteer::config::AuthType::ApiKey => {
            writeln!(&mut code, "        config.credentials.apiKey,")?;
            writeln!(&mut code, "        '', // No password for API key auth")?;
        }
        _ => {
            writeln!(&mut code, "        '', // TODO: Add credentials")?;
            writeln!(&mut code, "        ''")?;
        }
    }

    writeln!(&mut code, "        {{")?;
    writeln!(
        &mut code,
        "          usernameField: CssSelector('{}'),",
        config.login_username_selector
    )?;
    writeln!(
        &mut code,
        "          passwordField: CssSelector('{}'),",
        config.login_password_selector
    )?;
    writeln!(
        &mut code,
        "          submitButton: CssSelector('{}')",
        config.login_submit_selector
    )?;
    writeln!(&mut code, "        }}")?;
    writeln!(&mut code, "      ),")?;
    writeln!(
        &mut code,
        "      Scraper.waitForSelector(CssSelector('{}'))",
        config.login_success_selector
    )?;
    writeln!(&mut code, "    ),")?;

    Ok(code)
}

fn generate_extraction_program(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();

    writeln!(&mut code, "  buildExtractionProgram: (config) => {{")?;

    match config.workflow_type {
        WorkflowType::DomScraping => {
            generate_dom_extraction_program(&mut code, config)?;
        }
        WorkflowType::ApiScraping => {
            writeln!(&mut code, "    // API workflow doesn't use DOM extraction")?;
            writeln!(
                &mut code,
                "    return Scraper.extract(Extractors.text(CssSelector('body')));"
            )?;
        }
    }

    writeln!(&mut code, "  }},")?;

    Ok(code)
}

fn generate_dom_extraction_program(code: &mut String, config: &ScraperConfig) -> Result<()> {
    let item_selector = &config.dom_extraction.item_selector;

    if !config.navigation_steps.is_empty() {
        writeln!(code, "    const extractionSteps = [")?;

        for (i, step) in config.navigation_steps.iter().enumerate() {
            writeln!(code, "      Scraper.sequence(")?;
            writeln!(
                code,
                "        Scraper.navigate(config.{}Url as Url),",
                to_camel_case(&step.name.replace('-', "_"))
            )?;
            writeln!(
                code,
                "        Scraper.waitForSelector(CssSelector('{}')),",
                step.wait_value
            )?;

            if step.screenshot {
                writeln!(code, "        Scraper.screenshot('{}'),", step.name)?;
            }

            writeln!(code, "        Scraper.extractAll(")?;
            writeln!(code, "          CssSelector('{}'),", item_selector)?;
            writeln!(code, "{}", generate_composite_extractor(config)?)?;
            writeln!(code, "        )")?;
            write!(code, "      )")?;

            if i < config.navigation_steps.len() - 1 {
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
    } else {
        writeln!(code, "    return Scraper.extractAll(")?;
        writeln!(code, "      CssSelector('{}'),", item_selector)?;
        writeln!(code, "{}", generate_composite_extractor(config)?)?;
        writeln!(code, "    );")?;
    }

    Ok(())
}

fn generate_composite_extractor(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();
    let site_name = &config.site_name;
    let type_name = format!("{}Data", to_pascal_case(site_name));

    writeln!(&mut code, "          Extractors.composite<{}>({{", type_name)?;

    let field_count = config.dom_extraction.fields.len();
    for (i, (field_name, extraction_rule)) in config.dom_extraction.fields.iter().enumerate() {
        write!(&mut code, "            {}: ", field_name)?;

        match extraction_rule.extraction_type.as_str() {
            "text" => {
                write!(
                    &mut code,
                    "Extractors.text(CssSelector('{}'))",
                    extraction_rule.selector
                )?;

                if let Some(transform) = &extraction_rule.transform {
                    write!(&mut code, ".pipe(Scraper.map({}))", transform)?;
                }
            }
            "attribute" => {
                write!(
                    &mut code,
                    "Extractors.attribute(CssSelector('{}'), '{}')",
                    extraction_rule.selector,
                    extraction_rule.attribute.as_deref().unwrap_or("")
                )?;
            }
            "html" => {
                write!(
                    &mut code,
                    "Extractors.html(CssSelector('{}'))",
                    extraction_rule.selector
                )?;
            }
            _ => {
                write!(
                    &mut code,
                    "Extractors.text(CssSelector('{}'))",
                    extraction_rule.selector
                )?;
            }
        }

        if i < field_count - 1 {
            writeln!(&mut code, ",")?;
        } else {
            writeln!(&mut code)?;
        }
    }

    writeln!(&mut code, "          }})")?;

    Ok(code)
}

fn generate_process_data(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();
    let site_name = &config.site_name;
    let type_name = format!("{}Data", to_pascal_case(site_name));
    let schema_name = format!("{}DataSchema", to_pascal_case(site_name));

    writeln!(&mut code, "  processData: (raw: unknown) =>")?;
    writeln!(&mut code, "    Effect.gen(function* (_) {{")?;
    writeln!(&mut code, "      const parsed = yield* _(")?;
    writeln!(
        &mut code,
        "        Schema.decodeUnknown(Schema.Array({}))(raw),",
        schema_name
    )?;
    writeln!(
        &mut code,
        "        Effect.mapError(error => new ProcessingError('Data validation failed', error))"
    )?;
    writeln!(&mut code, "      );")?;
    writeln!(&mut code)?;

    writeln!(
        &mut code,
        "      yield* _(validateData<{}>(data => {{",
        type_name
    )?;
    writeln!(&mut code, "        const errors: string[] = [];")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "        if (data.length === 0) {{")?;
    writeln!(&mut code, "          errors.push('No data extracted');")?;
    writeln!(&mut code, "        }}")?;
    writeln!(&mut code)?;

    // Add custom validation rules
    for rule in &config.validation_rules {
        generate_validation_rule(&mut code, rule)?;
    }

    writeln!(
        &mut code,
        "        return {{ valid: errors.length === 0, errors }};"
    )?;
    writeln!(&mut code, "      }})(parsed));")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "      return parsed;")?;
    writeln!(&mut code, "    }}),")?;

    Ok(code)
}

fn generate_transform_data(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();
    let site_name = &config.site_name;
    let type_name = format!("{}Data", to_pascal_case(site_name));

    writeln!(
        &mut code,
        "  transformData: transformToParquet('{}', (item: {}) => ({{",
        site_name, type_name
    )?;
    writeln!(&mut code, "    ...item,")?;
    writeln!(&mut code, "    scrapedAt: new Date().toISOString(),")?;

    // Add custom transformations
    for (field, transform) in &config.transformations {
        writeln!(&mut code, "    {}: {},", field, transform)?;
    }

    writeln!(&mut code, "  }}))")?;

    Ok(code)
}

fn generate_validation_rule(
    code: &mut String,
    rule: &crate::puppeteer::config::ValidationRule,
) -> Result<()> {
    match rule.rule.as_str() {
        "required" => {
            writeln!(code, "        data.forEach((item, index) => {{")?;
            writeln!(code, "          if (!item.{}) {{", rule.field)?;
            writeln!(
                code,
                "            errors.push(`Item at index ${{index}}: {}`);}}"
            , rule.message)?;
            writeln!(code, "          }}")?;
            writeln!(code, "        }});")?;
        }
        "min" => {
            if let Some(value) = &rule.value {
                if let Some(num) = value.as_f64() {
                    writeln!(code, "        data.forEach((item, index) => {{")?;
                    writeln!(code, "          if (item.{} < {}) {{", rule.field, num)?;
                    writeln!(
                        code,
                        "            errors.push(`Item at index ${{index}}: {}`);}}"
                    , rule.message)?;
                    writeln!(code, "          }}")?;
                    writeln!(code, "        }});")?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn generate_api_config_schema(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();

    writeln!(&mut code, "const ConfigSchema = Schema.Struct({{")?;
    writeln!(
        &mut code,
        "  loginUrl: Schema.String.pipe(Schema.brand(\"Url\")),"
    )?;
    writeln!(
        &mut code,
        "  apiBaseUrl: Schema.String.pipe(Schema.brand(\"Url\")),"
    )?;

    writeln!(&mut code, "  credentials: Schema.Struct({{")?;

    match config.auth_type {
        crate::puppeteer::config::AuthType::Form => {
            writeln!(&mut code, "    username: Schema.String,")?;
            writeln!(&mut code, "    password: Schema.String")?;
        }
        crate::puppeteer::config::AuthType::ApiKey => {
            writeln!(&mut code, "    apiKey: Schema.String")?;
        }
        _ => {
            writeln!(&mut code, "    // TODO: Add auth fields")?;
        }
    }

    writeln!(&mut code, "  }})")?;
    writeln!(&mut code, "}});")?;
    writeln!(&mut code)?;
    writeln!(
        &mut code,
        "type Config = Schema.Schema.Type<typeof ConfigSchema>;"
    )?;

    Ok(code)
}

fn generate_api_login_program(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();

    writeln!(&mut code, "  buildLoginProgram: (config) =>")?;
    writeln!(&mut code, "    Scraper.sequence(")?;
    writeln!(&mut code, "      Scraper.navigate(config.loginUrl as Url),")?;
    writeln!(&mut code, "      login(")?;

    match config.auth_type {
        crate::puppeteer::config::AuthType::Form => {
            writeln!(&mut code, "        config.credentials.username,")?;
            writeln!(&mut code, "        config.credentials.password,")?;
        }
        crate::puppeteer::config::AuthType::ApiKey => {
            writeln!(&mut code, "        config.credentials.apiKey,")?;
            writeln!(&mut code, "        '', // No password for API key auth")?;
        }
        _ => {
            writeln!(&mut code, "        '',")?;
            writeln!(&mut code, "        ''")?;
        }
    }

    writeln!(&mut code, "        {{")?;
    writeln!(
        &mut code,
        "          usernameField: CssSelector('{}'),",
        config.login_username_selector
    )?;
    writeln!(
        &mut code,
        "          passwordField: CssSelector('{}'),",
        config.login_password_selector
    )?;
    writeln!(
        &mut code,
        "          submitButton: CssSelector('{}')",
        config.login_submit_selector
    )?;
    writeln!(&mut code, "        }}")?;
    writeln!(&mut code, "      ),")?;
    writeln!(
        &mut code,
        "      Scraper.waitForSelector(CssSelector('{}')),",
        config.login_success_selector
    )?;
    writeln!(&mut code, "      // Extract auth data")?;
    writeln!(&mut code, "      Scraper.extractAuth(")?;
    writeln!(&mut code, "        '{}',", config.auth_storage)?;
    writeln!(
        &mut code,
        "        ['{}', 'access_token', 'jwt']",
        config.auth_token_key
    )?;
    writeln!(&mut code, "      )")?;
    writeln!(&mut code, "    ),")?;

    Ok(code)
}

fn generate_api_requests(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();

    writeln!(&mut code, "  getAPIRequests: (): APIRequest[] => [")?;

    for (i, endpoint) in config.api_endpoints.iter().enumerate() {
        writeln!(&mut code, "    {{")?;
        writeln!(&mut code, "      method: '{}',", endpoint.method)?;
        writeln!(&mut code, "      endpoint: '{}',", endpoint.path)?;

        if let Some(params) = &endpoint.params {
            writeln!(
                &mut code,
                "      params: {},",
                serde_json::to_string(params).unwrap_or_default()
            )?;
        }

        if let Some(data) = &endpoint.data {
            writeln!(
                &mut code,
                "      data: {},",
                serde_json::to_string(data).unwrap_or_default()
            )?;
        }

        writeln!(&mut code, "      name: '{}'", endpoint.name)?;
        write!(&mut code, "    }}")?;

        if i < config.api_endpoints.len() - 1 {
            writeln!(&mut code, ",")?;
        } else {
            writeln!(&mut code)?;
        }
    }

    writeln!(&mut code, "  ],")?;

    Ok(code)
}

fn generate_api_response_processing(config: &ScraperConfig) -> Result<String> {
    let mut code = String::new();
    let site_name = &config.site_name;
    let type_name = format!("{}APIData", to_pascal_case(site_name));
    let schema_name = format!("{}DataSchema", to_pascal_case(site_name));

    writeln!(&mut code, "  processAPIResponses: (responses: APIResponse[]) =>")?;
    writeln!(&mut code, "    Effect.gen(function* (_) {{")?;
    writeln!(&mut code, "      const allData: {}[] = [];", type_name)?;
    writeln!(&mut code)?;

    writeln!(&mut code, "      for (const response of responses) {{")?;
    writeln!(&mut code, "        if (!response.data) {{")?;
    writeln!(
        &mut code,
        "          console.warn(`No data in response for ${{response.name}}`);"
    )?;
    writeln!(&mut code, "          continue;")?;
    writeln!(&mut code, "        }}")?;
    writeln!(&mut code)?;

    writeln!(&mut code, "        switch (response.name) {{")?;

    for (endpoint_name, mapping) in &config.api_extraction.response_mappings {
        writeln!(&mut code, "          case '{}': {{", endpoint_name)?;

        if mapping.data_path.is_empty() {
            writeln!(
                &mut code,
                "            const items = Array.isArray(response.data) ? response.data : [response.data];"
            )?;
        } else {
            writeln!(
                &mut code,
                "            const items = response.data.{} || [];",
                mapping.data_path
            )?;
        }

        if let Some(filter) = &mapping.filter {
            writeln!(
                &mut code,
                "            const filtered = items.filter({});",
                filter
            )?;
            writeln!(
                &mut code,
                "            allData.push(...filtered.map({}));",
                mapping.item_transform
            )?;
        } else {
            writeln!(
                &mut code,
                "            allData.push(...items.map({}));",
                mapping.item_transform
            )?;
        }

        writeln!(&mut code, "            break;")?;
        writeln!(&mut code, "          }}")?;
    }

    writeln!(&mut code, "          default:")?;
    writeln!(
        &mut code,
        "            console.warn(`No mapping for endpoint: ${{response.name}}`);"
    )?;
    writeln!(&mut code, "        }}")?;

    writeln!(&mut code, "      }}")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "      // Validate all data")?;
    writeln!(&mut code, "      const validated = yield* _(")?;
    writeln!(
        &mut code,
        "        Schema.decodeUnknown(Schema.Array({}))(allData),",
        schema_name
    )?;
    writeln!(
        &mut code,
        "        Effect.mapError(error => new ProcessingError('API data validation failed', error))"
    )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puppeteer::config::{get_dom_example, get_api_example};

    #[test]
    fn test_generate_scraper_definition_code() {
        let config = get_dom_example();
        let code = generate_scraper_definition_code(&config).unwrap();

        assert!(code.contains("ScraperDefinition"));
        assert!(code.contains("example_shopScraper"));  // Uses site_name directly
        assert!(code.contains("ConfigSchema"));
        assert!(code.contains("ExampleShopData"));  // Data type uses PascalCase
    }

    #[test]
    fn test_generate_api_scraper_definition_code() {
        let config = get_api_example();
        let code = generate_api_scraper_definition_code(&config).unwrap();

        assert!(code.contains("APIScraperDefinition"));
        assert!(code.contains("getAPIRequests"));
        assert!(code.contains("processAPIResponses"));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("example_shop"), "ExampleShop");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("example_shop"), "exampleShop");
    }
}

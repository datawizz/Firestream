use clap::{Parser, Subcommand};
use tera::Context;
use serde_json::Value;
use std::path::PathBuf;
use std::fs;

use templatizer_puppeteer::{example, generator};

#[derive(Parser)]
#[command(name = "fn-scraper-gen")]
#[command(about = "Generate functional web scrapers from configuration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a functional web scraper from configuration
    Generate {
        /// Path to JSON configuration file
        #[arg(short, long)]
        config: PathBuf,
        
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
        
        /// Use functional template (default: true)
        #[arg(long, default_value = "true")]
        functional: bool,
    },
    
    /// Show example configuration with extraction rules
    Example {
        /// Workflow type: dom or api
        #[arg(short, long, default_value = "dom")]
        workflow: String,
    },
    
    /// Validate a configuration file
    Validate {
        /// Path to JSON configuration file
        #[arg(short, long)]
        config: PathBuf,
    },
    
    /// List available template variables and extraction options
    ListVars,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Generate { config, output, functional } => {
            if *functional {
                generate_functional_scraper(config, output)?;
            } else {
                generate_legacy_scraper(config, output)?;
            }
        }
        Commands::Example { workflow } => {
            show_example(workflow)?;
        }
        Commands::Validate { config } => {
            validate_config(config)?;
        }
        Commands::ListVars => {
            list_variables()?;
        }
    }
    
    Ok(())
}

fn generate_functional_scraper(
    config_path: &PathBuf,
    output_dir: &PathBuf
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Generating functional web scraper...");
    
    // Read and parse configuration
    let config_content = fs::read_to_string(config_path)?;
    let config_json: Value = serde_json::from_str(&config_content)?;
    
    // Validate configuration
    generator::validate_functional_config(&config_json)?;
    
    // Render templates with generated code
    example::render_functional_templates_with_code(config_json.clone(), output_dir)?;
    
    // Get site name for next steps
    let site_name = config_json["site_name"]
        .as_str()
        .unwrap_or("scraper");
    
    println!("\n✅ Functional scraper generated successfully!");
    println!("\n📋 Next steps:");
    println!("  cd {}", output_dir.display());
    println!("  npm install");
    println!("  cp .env.example .env");
    println!("  # Edit .env with your credentials");
    println!("  npm run dev");
    println!("\n🐳 Docker:");
    println!("  docker build -t {}-scraper .", site_name);
    println!("\n✨ The scraper is fully functional - no manual implementation needed!");
    
    Ok(())
}

fn generate_legacy_scraper(
    config_path: &PathBuf,
    output_dir: &PathBuf
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating legacy web scraper...");
    
    // Read configuration
    let config_content = fs::read_to_string(config_path)?;
    let config_json: Value = serde_json::from_str(&config_content)?;
    let context = Context::from_serialize(config_json)?;
    
    // Render templates
    example::render_templates_to_directory(context.clone(), output_dir)?;
    
    println!("\nLegacy scraper generated successfully!");
    println!("Note: You'll need to manually implement the extraction logic.");
    
    Ok(())
}

fn show_example(workflow: &str) -> Result<(), Box<dyn std::error::Error>> {
    let example = match workflow {
        "dom" => generator::get_dom_example(),
        "api" => generator::get_api_example(),
        _ => return Err("Invalid workflow type. Use 'dom' or 'api'".into()),
    };
    
    println!("Example configuration for {} scraper:\n", workflow);
    println!("{}", serde_json::to_string_pretty(&example)?);
    
    Ok(())
}

fn validate_config(config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string(config_path)?;
    let config_json: Value = serde_json::from_str(&config_content)?;
    
    match generator::validate_functional_config(&config_json) {
        Ok(_) => {
            println!("✅ Configuration is valid!");
            
            // Show what will be generated
            let workflow = config_json["workflow_type"].as_str().unwrap_or("unknown");
            let fields = config_json["data_schema"]["fields"].as_array()
                .map(|f| f.len())
                .unwrap_or(0);
            
            println!("\nConfiguration summary:");
            println!("  Workflow: {}", workflow);
            println!("  Data fields: {}", fields);
            
            if workflow == "dom-scraping" {
                let steps = config_json["navigation_steps"].as_array()
                    .map(|s| s.len())
                    .unwrap_or(0);
                println!("  Navigation steps: {}", steps);
            } else if workflow == "api-scraping" {
                let endpoints = config_json["api_endpoints"].as_array()
                    .map(|e| e.len())
                    .unwrap_or(0);
                println!("  API endpoints: {}", endpoints);
            }
        }
        Err(e) => {
            println!("❌ Configuration validation failed:");
            println!("  {}", e);
        }
    }
    
    Ok(())
}

fn list_variables() -> Result<(), Box<dyn std::error::Error>> {
    println!("Available Template Variables for Functional Scrapers\n");
    println!("Core Configuration (same as legacy):");
    println!("  site_name, site_url, workflow_type, auth_type, etc.");
    println!("");
    println!("NEW: Data Schema Definition:");
    println!("  data_schema.fields[]");
    println!("    - name: Field name (e.g., 'id', 'title', 'price')");
    println!("    - type: TypeScript type ('string', 'number', 'boolean', 'Date')");
    println!("    - required: Whether field is required (default: false)");
    println!("    - description: Field description for documentation");
    println!("");
    println!("NEW: DOM Extraction Rules:");
    println!("  dom_extraction");
    println!("    - item_selector: CSS selector for items to extract");
    println!("    - pagination_selector: CSS selector for next page link (optional)");
    println!("    - fields: Extraction rules for each field");
    println!("      - type: 'text', 'attribute', 'html', 'composite'");
    println!("      - selector: CSS selector within item");
    println!("      - attribute: For 'attribute' type");
    println!("      - transform: JavaScript transform function (optional)");
    println!("");
    println!("NEW: API Extraction Rules:");
    println!("  api_extraction");
    println!("    - response_mappings: How to extract data from each endpoint");
    println!("      - [endpoint_name]");
    println!("        - data_path: Path to data in response (e.g., 'data.items')");
    println!("        - item_transform: Transform for each item");
    println!("        - filter: Filter expression (optional)");
    println!("");
    println!("NEW: Validation Rules:");
    println!("  validation_rules[]");
    println!("    - field: Field to validate");
    println!("    - rule: 'required', 'min', 'max', 'pattern', 'custom'");
    println!("    - value: Value for the rule");
    println!("    - message: Error message");
    println!("");
    println!("Example: Generate a fully functional scraper with:");
    println!("  fn-scraper-gen example --workflow dom > config.json");
    println!("  fn-scraper-gen generate -c config.json -o ./my-scraper");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    
    #[test]
    fn test_cli_parsing() {
        Cli::command().debug_assert();
    }
}

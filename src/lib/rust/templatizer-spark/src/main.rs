use clap::{Parser, Subcommand};
use tera::Context;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::fs;

mod example;

#[derive(Parser)]
#[command(name = "spark-templatizer")]
#[command(about = "Generate Spark applications from templates", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a Spark application from templates
    Generate {
        /// Template type: 'scala' or 'python'
        #[arg(short, long)]
        template: String,
        
        /// Path to JSON configuration file
        #[arg(short, long)]
        config: PathBuf,
        
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Show example configuration
    Example {
        /// Template type: 'scala' or 'python'
        #[arg(short, long)]
        template: String,
    },
    
    /// List available template variables
    ListVars {
        /// Template type: 'scala' or 'python'
        #[arg(short, long)]
        template: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Generate { template, config, output } => {
            generate_app(template, config, output)?;
        }
        Commands::Example { template } => {
            show_example(template)?;
        }
        Commands::ListVars { template } => {
            list_variables(template)?;
        }
    }
    
    Ok(())
}

fn generate_app(
    template_type: &str,
    config_path: &PathBuf,
    output_dir: &PathBuf
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating {} Spark application...", template_type);
    
    // Read configuration
    let config_content = fs::read_to_string(config_path)?;
    let config_json: Value = serde_json::from_str(&config_content)?;
    let context = Context::from_serialize(config_json)?;
    
    // Render templates
    example::render_templates_to_directory(template_type, context, output_dir)?;
    
    println!("\nApplication generated successfully!");
    println!("Next steps:");
    println!("  cd {}", output_dir.display());
    
    match template_type {
        "scala" => {
            println!("  sbt compile");
            println!("  sbt test");
            println!("  sbt assembly");
        }
        "python" => {
            println!("  python -m venv venv");
            println!("  source venv/bin/activate");
            println!("  pip install -r requirements.txt");
            println!("  pytest");
        }
        _ => {}
    }
    
    Ok(())
}

fn show_example(template_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let base_path = Path::new(&manifest_dir);
    
    let relative_path = match template_type {
        "scala" => "src/templates/spark_scala_boilerplate/example_context.json",
        "python" => "src/templates/spark_python_boilerplate/example_context.json",
        _ => return Err("Invalid template type. Use 'scala' or 'python'".into()),
    };
    
    let example_path = base_path.join(relative_path);
    
    let content = fs::read_to_string(example_path)?;
    println!("Example configuration for {} template:\n", template_type);
    println!("{}", content);
    
    Ok(())
}

fn list_variables(template_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let vars_path = match template_type {
        "scala" => "src/templates/spark_scala_boilerplate/TEMPLATE_VARIABLES.md",
        "python" => "src/templates/spark_python_boilerplate/TEMPLATE_VARIABLES.md",
        _ => return Err("Invalid template type. Use 'scala' or 'python'".into()),
    };
    
    let content = fs::read_to_string(vars_path)?;
    println!("{}", content);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    
    #[test]
    fn test_cli_parsing() {
        use clap::CommandFactory;
        
        Cli::command().debug_assert();
    }
}

//! Unified CLI for templatizer
//!
//! This module provides the command-line interface for all template generators.
//! It uses clap for argument parsing with subcommands for each generator type.

use crate::config::{GenerationOptions, OutputFormat, TemplateConfig};
use crate::embedded::{self, TemplateType};
use crate::error::{Result, TemplatizerError};
use crate::puppeteer::{PuppeteerGenerator, ScraperConfig, ScraperConfigBuilder, WorkflowType};
use crate::spark::{LanguageType, SparkAppConfig, SparkAppConfigBuilder, SparkGenerator};
use crate::superset::{DashboardConfig, SupersetConfigBuilder, SupersetGenerator, SupersetUploader};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

/// Unified template generator for Spark, Puppeteer, and Superset projects
#[derive(Parser, Debug)]
#[command(name = "templatizer")]
#[command(author = "Cogent Creation Co")]
#[command(version)]
#[command(about = "Generate project templates for data engineering workflows")]
#[command(long_about = r#"
Templatizer is a unified template generator for creating:
  - Spark applications (Python and Scala)
  - Puppeteer web scrapers
  - Superset dashboards

Each generator has its own subcommand with specific options.
Use --help on any subcommand for more details.
"#)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a Spark application
    #[command(alias = "sp")]
    Spark(SparkArgs),

    /// Generate a Puppeteer web scraper
    #[command(alias = "pup")]
    Puppeteer(PuppeteerArgs),

    /// Generate a Superset dashboard
    #[command(alias = "ss")]
    Superset(SupersetArgs),

    /// List available templates
    #[command(alias = "ls")]
    List(ListArgs),
}

/// Arguments for Spark template generation
#[derive(Parser, Debug)]
pub struct SparkArgs {
    /// Name of the project to generate
    #[arg(short, long)]
    pub name: String,

    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// Language variant (python or scala)
    #[arg(short, long, default_value = "python")]
    pub language: String,

    /// Configuration file (JSON or YAML)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Overwrite existing files
    #[arg(long)]
    pub overwrite: bool,

    /// Dry run - show what would be generated without writing
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for Puppeteer template generation
#[derive(Parser, Debug)]
pub struct PuppeteerArgs {
    /// Name of the scraper project
    #[arg(short, long)]
    pub name: String,

    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// Template variant (dom_scraper or fn_scraper)
    #[arg(short, long, default_value = "dom_scraper")]
    pub template: String,

    /// Target URL to scrape
    #[arg(long)]
    pub url: Option<String>,

    /// Configuration file (JSON or YAML)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Overwrite existing files
    #[arg(long)]
    pub overwrite: bool,

    /// Dry run - show what would be generated without writing
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for Superset template generation
#[derive(Parser, Debug)]
pub struct SupersetArgs {
    /// Name of the dashboard
    #[arg(short, long)]
    pub name: String,

    /// Output directory or ZIP file path
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// Configuration file (JSON or YAML)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Output as ZIP archive
    #[arg(long)]
    pub zip: bool,

    /// Upload to Superset server
    #[arg(long)]
    pub upload: bool,

    /// Superset server URL (required with --upload)
    #[arg(long)]
    pub server: Option<String>,

    /// Superset username (required with --upload)
    #[arg(long)]
    pub username: Option<String>,

    /// Superset password (required with --upload)
    #[arg(long)]
    pub password: Option<String>,

    /// Overwrite existing files
    #[arg(long)]
    pub overwrite: bool,

    /// Dry run - show what would be generated without writing
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for listing templates
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Filter by template type (spark, puppeteer, superset)
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Show detailed information
    #[arg(short, long)]
    pub detailed: bool,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

/// Execute the CLI command
pub fn run(cli: Cli) -> Result<()> {
    if cli.verbose {
        tracing::info!("Verbose mode enabled");
    }

    match cli.command {
        Commands::Spark(args) => run_spark(args, cli.verbose),
        Commands::Puppeteer(args) => run_puppeteer(args, cli.verbose),
        Commands::Superset(args) => run_superset(args, cli.verbose),
        Commands::List(args) => run_list(args, cli.verbose),
    }
}

fn run_spark(args: SparkArgs, verbose: bool) -> Result<()> {
    if verbose {
        info!("Generating Spark project: {}", args.name);
        info!("Language: {}", args.language);
        info!("Output: {}", args.output.display());
    }

    // Parse language type
    let language: LanguageType = args.language.parse().map_err(|e: String| {
        TemplatizerError::invalid_config(e)
    })?;

    // Load config from file or create default
    let config = if let Some(config_path) = &args.config {
        SparkAppConfig::from_json_file(config_path).or_else(|_| {
            SparkAppConfig::from_yaml_file(config_path)
        })?
    } else {
        SparkAppConfigBuilder::new(&args.name, "0.1.0")
            .language(language)
            .build()?
    };

    // Create generation options
    let options = GenerationOptions::new()
        .overwrite(args.overwrite)
        .verbose(verbose)
        .dry_run(args.dry_run);

    // Generate
    let generator = SparkGenerator::new()?;
    generator.generate_with_options(&config, &args.output, &options)?;

    if !args.dry_run {
        println!("Successfully generated Spark project at {}", args.output.display());
    }

    Ok(())
}

fn run_puppeteer(args: PuppeteerArgs, verbose: bool) -> Result<()> {
    if verbose {
        info!("Generating Puppeteer scraper: {}", args.name);
        info!("Template: {}", args.template);
        info!("Output: {}", args.output.display());
    }

    // Parse workflow type from template name
    let workflow_type = match args.template.as_str() {
        "dom_scraper" | "dom" => WorkflowType::DomScraping,
        "fn_scraper" | "api" | "api_scraper" => WorkflowType::ApiScraping,
        _ => {
            return Err(TemplatizerError::invalid_config(format!(
                "Unknown template type: {}. Use 'dom_scraper' or 'fn_scraper'",
                args.template
            )));
        }
    };

    // Load config from file or create default
    let config = if let Some(config_path) = &args.config {
        ScraperConfig::from_json_file(config_path).or_else(|_| {
            ScraperConfig::from_yaml_file(config_path)
        })?
    } else {
        let url = args.url.as_deref().unwrap_or("https://example.com");
        ScraperConfigBuilder::new(&args.name, url)
            .workflow_type(workflow_type)
            .item_selector(".item")
            .add_data_field("id", "string", true)
            .add_dom_extraction("id", "text", ".id", None, None)
            .build()?
    };

    // Create generation options
    let options = GenerationOptions::new()
        .overwrite(args.overwrite)
        .verbose(verbose)
        .dry_run(args.dry_run);

    // Generate
    let generator = PuppeteerGenerator::new()?;
    generator.generate_with_options(&config, &args.output, &options)?;

    if !args.dry_run {
        println!("Successfully generated Puppeteer scraper at {}", args.output.display());
    }

    Ok(())
}

fn run_superset(args: SupersetArgs, verbose: bool) -> Result<()> {
    if verbose {
        info!("Generating Superset dashboard: {}", args.name);
        info!("Output: {}", args.output.display());
        info!("ZIP: {}", args.zip);
    }

    // Load config from file or create minimal default
    let config = if let Some(config_path) = &args.config {
        DashboardConfig::from_json_file(config_path).or_else(|_| {
            DashboardConfig::from_yaml_file(config_path)
        })?
    } else {
        // Create a minimal config for testing
        use crate::superset::{Column, DatasetBuilder};

        SupersetConfigBuilder::new(&args.name, &args.name.to_lowercase().replace(' ', "-"))
            .database("PostgreSQL", "localhost", 5432, "database")
            .credentials("${DB_USERNAME}", "${DB_PASSWORD}")
            .add_dataset(
                DatasetBuilder::new("example_data", "example_table")
                    .add_column(Column {
                        name: "id".to_string(),
                        column_type: "INTEGER".to_string(),
                        ..Default::default()
                    })
                    .build()
            )
            .build()?
    };

    // Create generation options
    let format = if args.zip {
        OutputFormat::Zip
    } else {
        OutputFormat::Directory
    };

    let options = GenerationOptions::new()
        .format(format)
        .overwrite(args.overwrite)
        .verbose(verbose)
        .dry_run(args.dry_run);

    // Generate
    let generator = SupersetGenerator::new()?;
    generator.generate_with_options(&config, &args.output, &options)?;

    // Handle upload if requested
    if args.upload && !args.dry_run {
        let server = args.server.ok_or_else(|| {
            TemplatizerError::invalid_config("--server is required when using --upload")
        })?;

        info!("Uploading dashboard to {}", server);

        // Generate ZIP bytes for upload
        let zip_data = generator.generate_zip_bytes(&config)?;

        // Create runtime and upload
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            TemplatizerError::other(format!("Failed to create async runtime: {}", e))
        })?;

        rt.block_on(async {
            let mut uploader = if let (Some(username), Some(password)) = (&args.username, &args.password) {
                SupersetUploader::new(&server, username, password)?
            } else {
                SupersetUploader::from_env(&server)?
            };

            uploader.upload_dashboard(zip_data).await
        })?;

        println!("Successfully uploaded dashboard to {}", server);
    } else if !args.dry_run {
        println!("Successfully generated Superset dashboard at {}", args.output.display());
    }

    Ok(())
}

fn run_list(args: ListArgs, verbose: bool) -> Result<()> {
    let types: Vec<TemplateType> = if let Some(ref filter) = args.filter {
        match filter.to_lowercase().as_str() {
            "spark" | "sp" => vec![TemplateType::Spark],
            "puppeteer" | "pup" => vec![TemplateType::Puppeteer],
            "superset" | "ss" => vec![TemplateType::Superset],
            _ => {
                eprintln!("Unknown template type: {}", filter);
                eprintln!("Valid types: spark, puppeteer, superset");
                return Ok(());
            }
        }
    } else {
        vec![TemplateType::Spark, TemplateType::Puppeteer, TemplateType::Superset]
    };

    println!("Available templates:");
    println!();

    for template_type in types {
        println!("{}:", template_type.name().to_uppercase());

        let files = embedded::list_embedded_files_for_type(template_type);
        if files.is_empty() {
            println!("  (no templates embedded yet - run build first)");
        } else {
            if args.detailed {
                for file in &files {
                    println!("  - {}", file);
                }
            } else {
                // Group by subdirectory
                let mut subdirs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
                let prefix = template_type.prefix();
                for file in &files {
                    if let Some(rest) = file.strip_prefix(prefix) {
                        let rest = rest.trim_start_matches('/');
                        if let Some(subdir) = rest.split('/').next() {
                            subdirs.insert(subdir.to_string());
                        }
                    }
                }
                for subdir in subdirs {
                    println!("  - {}", subdir);
                }
            }
        }
        println!();
    }

    if verbose {
        println!("Total embedded files: {}", embedded::embedded_file_count());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_spark() {
        let args = vec!["templatizer", "spark", "-n", "my-app", "-l", "scala"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Spark(spark_args) => {
                assert_eq!(spark_args.name, "my-app");
                assert_eq!(spark_args.language, "scala");
            }
            _ => panic!("Expected Spark command"),
        }
    }

    #[test]
    fn test_cli_parse_puppeteer() {
        let args = vec!["templatizer", "puppeteer", "-n", "my-scraper", "-t", "fn_scraper"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Puppeteer(pup_args) => {
                assert_eq!(pup_args.name, "my-scraper");
                assert_eq!(pup_args.template, "fn_scraper");
            }
            _ => panic!("Expected Puppeteer command"),
        }
    }

    #[test]
    fn test_cli_parse_superset() {
        let args = vec!["templatizer", "superset", "-n", "my-dashboard", "--zip"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Superset(ss_args) => {
                assert_eq!(ss_args.name, "my-dashboard");
                assert!(ss_args.zip);
            }
            _ => panic!("Expected Superset command"),
        }
    }

    #[test]
    fn test_cli_parse_list() {
        let args = vec!["templatizer", "list", "-f", "spark"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::List(list_args) => {
                assert_eq!(list_args.filter, Some("spark".to_string()));
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_cli_aliases() {
        // Test spark alias
        let args = vec!["templatizer", "sp", "-n", "test"];
        assert!(Cli::try_parse_from(args).is_ok());

        // Test puppeteer alias
        let args = vec!["templatizer", "pup", "-n", "test"];
        assert!(Cli::try_parse_from(args).is_ok());

        // Test superset alias
        let args = vec!["templatizer", "ss", "-n", "test"];
        assert!(Cli::try_parse_from(args).is_ok());

        // Test list alias
        let args = vec!["templatizer", "ls"];
        assert!(Cli::try_parse_from(args).is_ok());
    }
}

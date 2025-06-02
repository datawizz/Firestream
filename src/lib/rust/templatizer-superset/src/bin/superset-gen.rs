//! CLI tool for Superset dashboard generation

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use templatizer_superset::{SupersetDashboardBuilder, SupersetUploader};
use tracing::{info, warn};

#[derive(Parser)]
#[command(
    name = "superset-gen",
    about = "Generate Apache Superset dashboards from JSON configurations",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate dashboard files to a directory
    Generate {
        /// Input JSON file
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Output directory
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
        
        /// Override database host
        #[arg(long)]
        db_host: Option<String>,
        
        /// Override database name
        #[arg(long)]
        db_name: Option<String>,
        
        /// Override database user
        #[arg(long)]
        db_user: Option<String>,
    },
    
    /// Generate dashboard ZIP file
    Zip {
        /// Input JSON file
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Output ZIP file
        #[arg(short, long, default_value = "dashboard.zip")]
        output: PathBuf,
    },
    
    /// Upload dashboard to Superset
    Upload {
        /// Input JSON file
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Superset base URL
        #[arg(short, long, default_value = "http://localhost:8088")]
        url: String,
        
        /// Superset username (or use SUPERSET_USERNAME env)
        #[arg(short = 'U', long)]
        username: Option<String>,
        
        /// Superset password (or use SUPERSET_PASSWORD env)
        #[arg(short = 'P', long)]
        password: Option<String>,
        
        /// Generate ZIP file before uploading
        #[arg(long)]
        save_zip: Option<PathBuf>,
    },
    
    /// Validate dashboard JSON configuration
    Validate {
        /// Input JSON file
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Check database connection
        #[arg(long)]
        check_db: bool,
    },
    
    /// Test connection to Superset
    TestConnection {
        /// Superset base URL
        #[arg(short, long, default_value = "http://localhost:8088")]
        url: String,
        
        /// List available databases
        #[arg(long)]
        list_databases: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    dotenv::dotenv().ok();
    
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();

    match cli.command {
        Commands::Generate { input, output, db_host, db_name, db_user } => {
            generate_dashboard(&input, &output, db_host, db_name, db_user)?;
        }
        
        Commands::Zip { input, output } => {
            generate_zip(&input, &output)?;
        }
        
        Commands::Upload { input, url, username, password, save_zip } => {
            upload_dashboard(&input, &url, username, password, save_zip).await?;
        }
        
        Commands::Validate { input, check_db } => {
            validate_config(&input, check_db)?;
        }
        
        Commands::TestConnection { url, list_databases } => {
            test_connection(&url, list_databases).await?;
        }
    }

    Ok(())
}

fn generate_dashboard(
    input: &PathBuf,
    output: &PathBuf,
    db_host: Option<String>,
    db_name: Option<String>,
    db_user: Option<String>,
) -> Result<()> {
    info!("Loading configuration from: {:?}", input);
    let json = std::fs::read_to_string(input)
        .context("Failed to read input file")?;

    let mut builder = SupersetDashboardBuilder::from_json(&json)?
        .with_database_from_env();

    // Apply overrides
    if let Some(host) = db_host {
        builder.config_mut().database.connection.host = host;
    }
    if let Some(name) = db_name {
        builder.config_mut().database.connection.database = name;
    }
    if let Some(user) = db_user {
        builder.config_mut().database.connection.username = user;
    }

    info!("Generating dashboard to: {:?}", output);
    builder.generate_to_directory(output)?;

    info!("✓ Dashboard generated successfully!");
    info!("  Database: {}", builder.config().database.name);
    info!("  Datasets: {}", builder.config().datasets.len());
    info!("  Charts: {}", builder.config().charts.len());
    
    Ok(())
}

fn generate_zip(input: &PathBuf, output: &PathBuf) -> Result<()> {
    info!("Loading configuration from: {:?}", input);
    let json = std::fs::read_to_string(input)
        .context("Failed to read input file")?;

    let builder = SupersetDashboardBuilder::from_json(&json)?
        .with_database_from_env();

    info!("Generating ZIP file...");
    let zip_data = builder.generate_zip()?;
    
    std::fs::write(output, zip_data)
        .context("Failed to write ZIP file")?;

    info!("✓ ZIP file generated: {:?}", output);
    Ok(())
}

async fn upload_dashboard(
    input: &PathBuf,
    url: &str,
    username: Option<String>,
    password: Option<String>,
    save_zip: Option<PathBuf>,
) -> Result<()> {
    info!("Loading configuration from: {:?}", input);
    let json = std::fs::read_to_string(input)
        .context("Failed to read input file")?;

    let builder = SupersetDashboardBuilder::from_json(&json)?
        .with_database_from_env();

    // Generate ZIP
    info!("Generating dashboard ZIP...");
    let zip_data = builder.generate_zip()?;

    // Save ZIP if requested
    if let Some(zip_path) = save_zip {
        std::fs::write(&zip_path, &zip_data)
            .context("Failed to save ZIP file")?;
        info!("✓ Saved ZIP to: {:?}", zip_path);
    }

    // Set credentials if provided
    if let (Some(user), Some(pass)) = (username, password) {
        std::env::set_var("SUPERSET_USERNAME", user);
        std::env::set_var("SUPERSET_PASSWORD", pass);
    }

    // Upload
    info!("Uploading to Superset at: {}", url);
    let mut uploader = SupersetUploader::from_env(url)
        .context("Failed to create uploader - check SUPERSET_USERNAME and SUPERSET_PASSWORD")?;
    
    uploader.upload_dashboard(zip_data).await?;
    info!("✓ Dashboard uploaded successfully!");

    Ok(())
}

fn validate_config(input: &PathBuf, check_db: bool) -> Result<()> {
    info!("Validating configuration from: {:?}", input);
    let json = std::fs::read_to_string(input)
        .context("Failed to read input file")?;

    let builder = SupersetDashboardBuilder::from_json(&json)?;
    let config = builder.config();

    // Basic validation happens during parsing
    info!("✓ Configuration is valid!");
    info!("  Title: {}", config.dashboard.title);
    info!("  Database: {}", config.database.name);
    info!("  Connection: {}", config.get_safe_sqlalchemy_uri());
    info!("  Datasets: {}", config.datasets.len());
    for dataset in &config.datasets {
        info!("    - {} ({} columns, {} metrics)", 
            dataset.name, dataset.columns.len(), dataset.metrics.len());
    }
    info!("  Charts: {}", config.charts.len());
    for chart in &config.charts {
        info!("    - {} ({})", chart.title, chart.chart_type);
    }

    if check_db {
        warn!("Database connection check not implemented yet");
    }

    Ok(())
}

async fn test_connection(url: &str, list_databases: bool) -> Result<()> {
    info!("Testing connection to Superset at: {}", url);

    let mut uploader = SupersetUploader::from_env(url)
        .context("Failed to create uploader - check SUPERSET_USERNAME and SUPERSET_PASSWORD")?;

    match uploader.test_connection().await? {
        true => {
            info!("✓ Successfully connected to Superset!");
            
            if list_databases {
                info!("Fetching database list...");
                let databases = uploader.list_databases().await?;
                info!("Available databases:");
                for db in databases {
                    info!("  - {}", db);
                }
            }
        }
        false => {
            warn!("✗ Failed to connect to Superset");
            warn!("  Check your credentials and URL");
        }
    }

    Ok(())
}

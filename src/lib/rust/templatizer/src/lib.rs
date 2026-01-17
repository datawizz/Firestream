//! Unified template generator for Spark, Puppeteer, and Superset projects
//!
//! This crate provides a single, unified interface for generating project templates
//! across multiple target platforms used in data engineering workflows.
//!
//! # Overview
//!
//! Templatizer consolidates three template generators:
//!
//! - **Spark**: Generate Python (PySpark) or Scala Spark applications
//! - **Puppeteer**: Generate Node.js web scrapers for Airflow/Kubernetes
//! - **Superset**: Generate Apache Superset dashboard configurations
//!
//! # Architecture
//!
//! The crate uses [workspace-embed](../workspace_embed) to embed template files
//! at compile time from `src/templates/` in the repository root. At runtime,
//! templates are extracted to a temporary directory and processed with [Tera](tera).
//!
//! # Example Usage
//!
//! ## As a library
//!
//! ```rust,ignore
//! use templatizer::{TemplateEngine, TemplateType};
//! use tera::Context;
//!
//! // Create an engine for Spark templates
//! let engine = TemplateEngine::new(TemplateType::Spark)?;
//!
//! // List available templates
//! for template in engine.list_templates() {
//!     println!("Template: {}", template);
//! }
//!
//! // Render a template
//! let mut context = Context::new();
//! context.insert("project_name", "my-spark-app");
//! let output = engine.render("main.py.tera", &context)?;
//! ```
//!
//! ## Generator Usage
//!
//! ```rust,ignore
//! // Spark generator
//! use templatizer::spark::{SparkAppConfigBuilder, LanguageType, SparkGenerator};
//!
//! let config = SparkAppConfigBuilder::new("my-spark-app", "1.0.0")
//!     .language(LanguageType::Python)
//!     .s3_config("my-bucket", "us-east-1")
//!     .build()?;
//!
//! let generator = SparkGenerator::new()?;
//! generator.generate(&config, "/output/path".as_ref())?;
//!
//! // Puppeteer generator
//! use templatizer::puppeteer::{ScraperConfigBuilder, WorkflowType, PuppeteerGenerator};
//!
//! let config = ScraperConfigBuilder::new("my-scraper", "https://example.com")
//!     .workflow_type(WorkflowType::DomScraping)
//!     .item_selector(".product-item")
//!     .add_data_field("id", "string", true)
//!     .add_dom_extraction("id", "attribute", "[data-id]", Some("data-id"), None)
//!     .build()?;
//!
//! let generator = PuppeteerGenerator::new()?;
//! generator.generate(&config, "/output/path".as_ref())?;
//!
//! // Superset generator
//! use templatizer::superset::{SupersetConfigBuilder, SupersetGenerator, DatasetBuilder, Column};
//!
//! let config = SupersetConfigBuilder::new("Sales Dashboard", "sales-dashboard")
//!     .database("PostgreSQL", "localhost", 5432, "analytics")
//!     .credentials("user", "password")
//!     .add_dataset(
//!         DatasetBuilder::new("sales", "sales_table")
//!             .time_column("date")
//!             .build()
//!     )
//!     .build()?;
//!
//! let generator = SupersetGenerator::new()?;
//! generator.generate(&config, "/output/path".as_ref())?;
//! ```
//!
//! ## As a CLI
//!
//! ```bash
//! # Generate a Spark application
//! templatizer spark -n my-app -l python -o ./output
//!
//! # Generate a Puppeteer scraper
//! templatizer puppeteer -n my-scraper -t dom_scraper -o ./output
//!
//! # Generate a Superset dashboard
//! templatizer superset -n my-dashboard -c config.yaml --zip -o ./dashboard.zip
//!
//! # List available templates
//! templatizer list
//! ```
//!
//! # Module Structure
//!
//! - [`embedded`]: Template embedding and extraction using rust-embed
//! - [`engine`]: Unified Tera template engine wrapper
//! - [`config`]: Shared configuration traits and types
//! - [`error`]: Error types for the crate
//! - [`cli`]: Command-line interface implementation
//! - [`puppeteer`]: Puppeteer web scraper generator
//! - [`spark`]: Spark application generator (Python and Scala)
//! - [`superset`]: Superset dashboard generator with upload support

pub mod cli;
pub mod compat;
pub mod config;
pub mod embedded;
pub mod engine;
pub mod error;
pub mod puppeteer;
pub mod spark;
pub mod superset;

// Re-export key types for convenient access
pub use config::{ConfigBuilder, GenerationOptions, OutputFormat, TemplateConfig};
pub use embedded::{
    extract_templates, get_embedded_file, get_embedded_file_str, has_template,
    list_embedded_files, list_templates, template_path, templates_dir, EmbeddedTemplates,
    ExtractedWorkspace, TemplateType,
};
pub use engine::TemplateEngine;
pub use error::{Result, TemplatizerError};

// Re-export generators for convenient access
pub use puppeteer::PuppeteerGenerator;
pub use spark::SparkGenerator;
pub use superset::SupersetGenerator;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the number of embedded template files
pub fn embedded_count() -> usize {
    embedded::embedded_file_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_template_types() {
        assert_eq!(TemplateType::Spark.name(), "spark");
        assert_eq!(TemplateType::Puppeteer.name(), "puppeteer");
        assert_eq!(TemplateType::Superset.name(), "superset");
    }
}

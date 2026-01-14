//! Puppeteer web scraper template generator
//!
//! This module provides functionality for generating production-ready
//! Puppeteer-based web scrapers for Airflow/Kubernetes deployments.
//!
//! Templates available:
//! - `dom_scraper`: DOM-based scraping with Puppeteer
//! - `fn_scraper`: Function-based scraping pattern with API support
//!
//! # Example
//!
//! ```rust,ignore
//! use templatizer::puppeteer::{ScraperConfig, ScraperConfigBuilder, WorkflowType, PuppeteerGenerator};
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
//! ```

pub mod config;
pub mod generator;

// Re-export key types
pub use config::{
    ApiEndpoint, ApiExtraction, AuthType, DataField, DataSchema, DomExtraction, DomExtractionRule,
    NavigationStep, ResponseMapping, ScraperConfig, ScraperConfigBuilder, ValidationRule,
    WorkflowType, get_api_example, get_dom_example,
};

pub use generator::{
    PuppeteerGenerator, generate_api_scraper_definition_code, generate_scraper_definition_code,
};

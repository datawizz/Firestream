//! # Superset Dashboard Generator
//!
//! An elegant library for generating Apache Superset dashboards from JSON configurations.
//! 
//! This crate provides a high-level API for creating complete Superset dashboard packages
//! including databases, datasets, charts, and layouts from a simple JSON specification.
//!
//! ## Features
//! 
//! - Elegant JSON-based dashboard configuration
//! - PostgreSQL-optimized with sensible defaults
//! - Automatic UUID generation and relationship management
//! - Tera-based YAML templating for flexibility
//! - Built-in ZIP packaging
//! - Direct upload to Superset via REST API
//! - Environment-based credential management

pub mod config;
pub mod generator;
pub mod models;
pub mod templates;
pub mod uploader;
pub mod utils;

use anyhow::Result;
use std::path::Path;

pub use config::{DashboardConfig, ConfigBuilder};
pub use generator::DashboardGenerator;
pub use models::*;
pub use uploader::SupersetUploader;
pub use utils::{SqlBuilder, validate_virtual_dataset_sql};

/// Main entry point for dashboard generation
pub struct SupersetDashboardBuilder {
    config: DashboardConfig,
    generator: DashboardGenerator,
}

impl SupersetDashboardBuilder {
    /// Create a new dashboard builder from a JSON configuration
    pub fn from_json(json: &str) -> Result<Self> {
        let config: DashboardConfig = serde_json::from_str(json)?;
        let generator = DashboardGenerator::new()?;
        
        Ok(Self { config, generator })
    }

    /// Create a new dashboard builder from a configuration object
    pub fn from_config(config: DashboardConfig) -> Result<Self> {
        let generator = DashboardGenerator::new()?;
        Ok(Self { config, generator })
    }

    /// Generate the dashboard files to a directory
    pub fn generate_to_directory(&self, output_dir: &Path) -> Result<()> {
        self.generator.generate(&self.config, output_dir)
    }

    /// Generate and package the dashboard as a ZIP file
    pub fn generate_zip(&self) -> Result<Vec<u8>> {
        self.generator.generate_zip(&self.config)
    }

    /// Generate and upload the dashboard to Superset
    pub async fn upload_to_superset(&self, base_url: &str) -> Result<()> {
        let zip_data = self.generate_zip()?;
        let mut uploader = SupersetUploader::from_env(base_url)?;
        uploader.upload_dashboard(zip_data).await
    }

    /// Builder pattern: set database connection from environment
    pub fn with_database_from_env(mut self) -> Self {
        self.config.database.connection.username = 
            std::env::var("POSTGRES_USER").unwrap_or_else(|_| self.config.database.connection.username);
        self.config.database.connection.password = 
            std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| self.config.database.connection.password);
        self.config.database.connection.host = 
            std::env::var("POSTGRES_HOST").unwrap_or_else(|_| self.config.database.connection.host);
        self
    }

    /// Builder pattern: add a dataset
    pub fn add_dataset(mut self, dataset: Dataset) -> Self {
        self.config.datasets.push(dataset);
        self
    }

    /// Builder pattern: add a chart
    pub fn add_chart(mut self, chart: Chart) -> Self {
        self.config.charts.push(chart);
        self
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &DashboardConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut DashboardConfig {
        &mut self.config
    }
}

/// Convenience function to generate a dashboard from JSON
pub async fn generate_dashboard_from_json(json: &str, superset_url: &str) -> Result<()> {
    let builder = SupersetDashboardBuilder::from_json(json)?
        .with_database_from_env();
    
    builder.upload_to_superset(superset_url).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let json = r#"{
            "dashboard": {
                "title": "Test Dashboard",
                "slug": "test-dashboard"
            },
            "database": {
                "name": "Test DB",
                "connection": {
                    "host": "localhost",
                    "port": 5432,
                    "database": "test"
                }
            },
            "datasets": [],
            "charts": [],
            "layout": {
                "type": "grid",
                "width": 12,
                "rows": []
            }
        }"#;

        let builder = SupersetDashboardBuilder::from_json(json).unwrap();
        assert_eq!(builder.config().dashboard.title, "Test Dashboard");
    }
}

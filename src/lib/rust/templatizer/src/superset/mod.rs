//! Superset dashboard template generator
//!
//! This module provides functionality for generating Apache Superset
//! dashboard configurations from JSON/YAML definitions.
//!
//! Templates available:
//! - `dashboard.yaml.tera`: Dashboard definition
//! - `chart.yaml.tera`: Chart/visualization definitions
//! - `dataset.yaml.tera`: Dataset configurations
//! - `database.yaml.tera`: Database connection configs
//! - `metadata.yaml.tera`: Export metadata
//!
//! # Example
//!
//! ```rust,ignore
//! use templatizer::superset::{
//!     DashboardConfig, SupersetConfigBuilder, SupersetGenerator, SupersetUploader,
//!     Dataset, Chart, ChartBuilder, DatasetBuilder, Column,
//! };
//!
//! // Build configuration
//! let config = SupersetConfigBuilder::new("Sales Dashboard", "sales-dashboard")
//!     .database("PostgreSQL", "localhost", 5432, "analytics")
//!     .credentials("user", "password")
//!     .add_dataset(
//!         DatasetBuilder::new("sales_data", "sales")
//!             .time_column("order_date")
//!             .add_column(Column {
//!                 name: "revenue".to_string(),
//!                 column_type: "DECIMAL".to_string(),
//!                 ..Default::default()
//!             })
//!             .build()
//!     )
//!     .add_chart(
//!         ChartBuilder::new("revenue_chart", "Revenue Over Time", "line")
//!             .dataset("sales_data")
//!             .add_metric("SUM(revenue)")
//!             .time_config("order_date", Some("month"), "last 12 months")
//!             .build()
//!     )
//!     .build()?;
//!
//! // Generate to directory
//! let generator = SupersetGenerator::new()?;
//! generator.generate(&config, "/output/path".as_ref())?;
//!
//! // Or generate and upload directly
//! let zip_data = generator.generate_zip_bytes(&config)?;
//! let mut uploader = SupersetUploader::from_env("http://localhost:8088")?;
//! uploader.upload_dashboard(zip_data).await?;
//! ```

pub mod config;
pub mod generator;
pub mod models;
pub mod uploader;
pub mod utils;

// Re-export key types from models
pub use models::{
    AccessConfig, CertificationConfig, Chart, ChartBuilder, ChartConfig, ChartFilter, Column,
    DashboardConfig, DashboardLayout, DashboardMetadata, DatabaseConfig, DatabaseOptions,
    Dataset, DatasetBuilder, FilterConfig, FilterTarget, LayoutComponent, LayoutRow, Metric,
    NativeFilter, PostgresConnection, TimeConfig,
};

// Re-export config builder
pub use config::SupersetConfigBuilder;

// Re-export generator
pub use generator::SupersetGenerator;

// Re-export uploader
pub use uploader::{SupersetUploader, UploaderBuilder};

// Re-export utilities
pub use utils::{
    build_metric_expression, default_chart_options, format_time_grain, get_number_format,
    map_chart_type, normalize_column_type, parse_dimension_reference, parse_time_range,
    safe_identifier, sanitize_filename, uuid_from_string, validate_sql_expression,
    validate_virtual_dataset_sql, SqlBuilder,
};

//! Iceberg Manager - DataFusion-centric Apache Iceberg integration
//! 
//! This crate provides seamless integration between Apache Iceberg and DataFusion,
//! allowing you to query Iceberg tables using SQL through DataFusion's query engine.
//! 
//! # Features
//! 
//! - REST and Memory catalog support
//! - Direct SQL querying of Iceberg tables
//! - Integration with DataFusion's catalog system
//! - Table creation and management
//! - Schema inspection and data preview

pub mod catalog_provider;
pub mod error;
pub mod schema_converter;
pub mod table_manager;
pub mod tpc_seeder;
pub mod types;

pub use error::{Error, Result};
pub use table_manager::{CatalogConfig, TableManager, TableSchema, TableStats, TablePreview, TableField, create_test_schema};
pub use tpc_seeder::{TpcSeeder, TpcSeederConfig, TpcBenchmark, TableInfo};
pub use schema_converter::SchemaConverter;

// Re-export commonly used types from official iceberg crate
pub use iceberg::spec::{
    Schema,
    Type, 
    PrimitiveType, 
    NestedField,
    UnboundPartitionSpec, 
    Transform,
};

// Re-export DataFusion types for convenience
pub use datafusion::prelude::{SessionContext, DataFrame};

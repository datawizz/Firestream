//! Spark application template generator
//!
//! This module provides functionality for generating Spark applications
//! in both Python (PySpark) and Scala variants.
//!
//! Templates available:
//! - `python`: PySpark application templates
//! - `scala`: Scala Spark application templates
//!
//! # Example
//!
//! ```rust,ignore
//! use templatizer::spark::{SparkAppConfig, SparkAppConfigBuilder, LanguageType, SparkGenerator};
//!
//! let config = SparkAppConfigBuilder::new("my-spark-app", "1.0.0")
//!     .language(LanguageType::Python)
//!     .organization("com.example")
//!     .s3_config("my-bucket", "us-east-1")
//!     .enable_delta()
//!     .build()?;
//!
//! let generator = SparkGenerator::new()?;
//! generator.generate(&config, "/output/path".as_ref())?;
//! ```

pub mod config;
pub mod generator;

// Re-export key types
pub use config::{
    LanguageType, SparkAppConfig, SparkAppConfigBuilder, get_python_example, get_scala_example,
};

pub use generator::SparkGenerator;

//! Shared configuration traits for template generators
//!
//! This module defines the common interface that all template configurations
//! must implement, enabling a unified approach to template generation.

use crate::error::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use tera::Context;

/// Trait for template configuration types
///
/// All generator-specific configuration types (SparkConfig, PuppeteerConfig,
/// SupersetConfig) should implement this trait to enable unified handling.
pub trait TemplateConfig: Serialize + DeserializeOwned + Clone {
    /// Get the name of the project/output being generated
    fn project_name(&self) -> &str;

    /// Validate the configuration
    ///
    /// Returns Ok(()) if the configuration is valid, or an error describing
    /// what is invalid.
    fn validate(&self) -> Result<()>;

    /// Convert this configuration to a Tera context for rendering
    fn to_context(&self) -> Result<Context> {
        let value = serde_json::to_value(self)?;
        let context = Context::from_value(value)?;
        Ok(context)
    }

    /// Load configuration from a JSON file
    fn from_json_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a YAML file
    fn from_yaml_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a JSON file
    fn to_json_file(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save configuration to a YAML file
    fn to_yaml_file(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Builder trait for creating configurations fluently
///
/// Generators can implement this to provide a builder pattern for
/// constructing configurations programmatically.
pub trait ConfigBuilder: Sized {
    /// The configuration type this builder creates
    type Config: TemplateConfig;

    /// Create a new builder with default values
    fn new() -> Self;

    /// Set the project name
    fn project_name(self, name: impl Into<String>) -> Self;

    /// Build the final configuration
    ///
    /// This should validate the configuration before returning it.
    fn build(self) -> Result<Self::Config>;
}

/// Output format for generated content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Output to a directory structure
    #[default]
    Directory,
    /// Output to a ZIP archive
    Zip,
    /// Output to stdout (single file templates only)
    Stdout,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Directory => write!(f, "directory"),
            OutputFormat::Zip => write!(f, "zip"),
            OutputFormat::Stdout => write!(f, "stdout"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "directory" | "dir" => Ok(OutputFormat::Directory),
            "zip" => Ok(OutputFormat::Zip),
            "stdout" | "-" => Ok(OutputFormat::Stdout),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

/// Generation options common to all generators
#[derive(Debug, Clone)]
pub struct GenerationOptions {
    /// Output format
    pub format: OutputFormat,
    /// Whether to overwrite existing files
    pub overwrite: bool,
    /// Whether to print verbose output
    pub verbose: bool,
    /// Dry run mode - don't actually write files
    pub dry_run: bool,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Directory,
            overwrite: false,
            verbose: false,
            dry_run: false,
        }
    }
}

impl GenerationOptions {
    /// Create new generation options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the output format
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Enable overwrite mode
    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    /// Enable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable dry run mode
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Directory.to_string(), "directory");
        assert_eq!(OutputFormat::Zip.to_string(), "zip");
        assert_eq!(OutputFormat::Stdout.to_string(), "stdout");
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("directory".parse::<OutputFormat>().unwrap(), OutputFormat::Directory);
        assert_eq!("dir".parse::<OutputFormat>().unwrap(), OutputFormat::Directory);
        assert_eq!("zip".parse::<OutputFormat>().unwrap(), OutputFormat::Zip);
        assert_eq!("stdout".parse::<OutputFormat>().unwrap(), OutputFormat::Stdout);
        assert_eq!("-".parse::<OutputFormat>().unwrap(), OutputFormat::Stdout);
    }

    #[test]
    fn test_generation_options_default() {
        let opts = GenerationOptions::default();
        assert_eq!(opts.format, OutputFormat::Directory);
        assert!(!opts.overwrite);
        assert!(!opts.verbose);
        assert!(!opts.dry_run);
    }

    #[test]
    fn test_generation_options_builder() {
        let opts = GenerationOptions::new()
            .format(OutputFormat::Zip)
            .overwrite(true)
            .verbose(true)
            .dry_run(true);

        assert_eq!(opts.format, OutputFormat::Zip);
        assert!(opts.overwrite);
        assert!(opts.verbose);
        assert!(opts.dry_run);
    }
}

//! Test generation module
//!
//! Generates Goss YAML files from templates and specifications.

pub mod goss_yaml;
pub mod templates;

pub use goss_yaml::GossYamlGenerator;
pub use templates::TemplateEngine;

/// Error type for generator operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Template error: {0}")]
    Template(String),

    #[error("Rendering failed: {0}")]
    RenderFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tera error: {0}")]
    Tera(#[from] tera::Error),
}

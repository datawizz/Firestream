//! Example module for rendering templates

use tera::Context;
use std::path::Path;
use crate::{DomScraperTemplatizer, FunctionalScraperTemplatizer};

/// Render legacy templates to directory
pub fn render_templates_to_directory(
    context: Context,
    output_dir: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let mut templatizer = DomScraperTemplatizer::new()?;
    templatizer.render_to_directory(context, output_dir)
}

/// Render functional templates to directory
pub fn render_functional_templates_to_directory(
    context: Context,
    output_dir: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let mut templatizer = FunctionalScraperTemplatizer::new()?;
    templatizer.render_to_directory(context, output_dir)
}

/// Render functional templates with generated code
pub fn render_functional_templates_with_code(
    config: serde_json::Value,
    output_dir: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let mut templatizer = FunctionalScraperTemplatizer::new()?;
    templatizer.render_with_generated_code(config, output_dir)
}

pub mod engine;
pub mod processor;
pub mod prompts;
pub mod context;
pub mod embedded;

pub use processor::TemplateProcessor;
pub use prompts::ProjectConfig;

use crate::core::Result;
use std::path::PathBuf;

/// Execute the template command
pub async fn execute_template(
    name: Option<String>,
    project_type: &str,
    output: &PathBuf,
    non_interactive: bool,
    values: Option<&PathBuf>,
) -> Result<()> {
    // Get project configuration
    let config = if non_interactive {
        if let Some(values_path) = values {
            ProjectConfig::from_values_file(values_path)?
        } else {
            ProjectConfig::default_with_name(name)
        }
    } else {
        ProjectConfig::from_interactive(name)?
    };
    
    // Validate project type
    match project_type {
        "python-fastapi" | "helm" | "kubernetes" | "docker" => {},
        _ => return Err(crate::core::FirestreamError::GeneralError(
            format!("Unsupported project type: {}", project_type)
        )),
    }
    
    // Process templates
    let processor = TemplateProcessor::new(output.clone())?;
    processor.process(&config, project_type).await?;
    
    println!("✨ Project '{}' created successfully!", config.project_name);
    println!("\nNext steps:");
    println!("  1. cd {}", output.join(&config.project_name).display());
    println!("  2. Review the generated files");
    println!("  3. Run 'make build' to build the Docker image");
    println!("  4. Run 'firestream deploy' to deploy to Kubernetes");
    
    Ok(())
}

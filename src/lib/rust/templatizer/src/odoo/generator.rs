//! Generator for Odoo addons project templates

use crate::config::{GenerationOptions, TemplateConfig};
use crate::embedded::TemplateType;
use crate::engine::TemplateEngine;
use crate::error::{Result, TemplatizerError};
use crate::odoo::config::OdooConfig;
use std::path::Path;
use tracing::{debug, info};

/// Generator for Odoo addons projects
pub struct OdooGenerator {
    engine: TemplateEngine,
}

impl OdooGenerator {
    /// Create a new Odoo generator
    pub fn new() -> Result<Self> {
        let engine = TemplateEngine::new(TemplateType::Odoo)?;
        Ok(Self { engine })
    }

    /// Generate an Odoo project to a directory
    pub fn generate(&self, config: &OdooConfig, output_dir: &Path) -> Result<()> {
        self.generate_with_options(config, output_dir, &GenerationOptions::default())
    }

    /// Generate with specific options
    pub fn generate_with_options(
        &self,
        config: &OdooConfig,
        output_dir: &Path,
        options: &GenerationOptions,
    ) -> Result<()> {
        config.validate()?;

        info!(
            "Generating Odoo {} addons project: {}",
            config.odoo_version, config.project_name
        );

        if options.dry_run {
            info!("Dry run — would generate to: {}", output_dir.display());
            return Ok(());
        }

        // Create output directory
        std::fs::create_dir_all(output_dir).map_err(|e| {
            TemplatizerError::output_error(output_dir.to_path_buf(), e.to_string())
        })?;

        // Convert config to context
        let context = config.to_context()?;

        // Render all .tera templates
        for template_name in self.engine.list_templates() {
            // Determine output path: strip .tera extension
            let output_file = template_name.trim_end_matches(".tera");
            let output_path = output_dir.join(output_file);

            // Check for existing file
            if output_path.exists() && !options.overwrite {
                debug!("Skipping existing file: {}", output_path.display());
                continue;
            }

            // render_to_file handles parent directory creation
            self.engine
                .render_to_file(&template_name, &context, &output_path)?;

            if options.verbose {
                info!("Generated: {}", output_path.display());
            }
        }

        // Copy non-.tera files (e.g., .gitkeep)
        self.copy_static_files(output_dir, options)?;

        // Make scripts executable
        Self::make_executable(output_dir)?;

        info!(
            "Successfully generated Odoo addons project to {}",
            output_dir.display()
        );
        Ok(())
    }

    /// Copy non-.tera files from the template directory to the output
    fn copy_static_files(&self, output_dir: &Path, options: &GenerationOptions) -> Result<()> {
        let templates_root = self.engine.templates_root();
        if !templates_root.exists() {
            return Ok(());
        }
        Self::copy_static_recursive(&templates_root, &templates_root, output_dir, options)
    }

    fn copy_static_recursive(
        base: &Path,
        dir: &Path,
        output_dir: &Path,
        options: &GenerationOptions,
    ) -> Result<()> {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries {
            let entry = entry.map_err(|e| {
                TemplatizerError::other(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_dir() {
                Self::copy_static_recursive(base, &path, output_dir, options)?;
            } else if path.is_file() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                // Skip .tera files (already handled by the engine)
                // Skip example_context.json and TEMPLATE_VARIABLES.md (meta files)
                if name.ends_with(".tera")
                    || name == "example_context.json"
                    || name == "TEMPLATE_VARIABLES.md"
                {
                    continue;
                }

                // Compute relative path from template root
                let rel_path = path
                    .strip_prefix(base)
                    .map_err(|e| TemplatizerError::other(e.to_string()))?;
                let dest = output_dir.join(rel_path);

                if dest.exists() && !options.overwrite {
                    debug!("Skipping existing static file: {}", dest.display());
                    continue;
                }

                // Create parent dirs and copy
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        TemplatizerError::output_error(parent.to_path_buf(), e.to_string())
                    })?;
                }
                std::fs::copy(&path, &dest).map_err(|e| {
                    TemplatizerError::output_error(dest.clone(), e.to_string())
                })?;

                if options.verbose {
                    info!("Copied: {}", dest.display());
                }
            }
        }

        Ok(())
    }

    /// Make script files executable
    fn make_executable(output_dir: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let scripts_dir = output_dir.join("scripts");
            if scripts_dir.exists() {
                for entry in std::fs::read_dir(&scripts_dir).map_err(|e| {
                    TemplatizerError::other(format!("Failed to read scripts dir: {}", e))
                })? {
                    let entry = entry.map_err(|e| {
                        TemplatizerError::other(format!("Failed to read entry: {}", e))
                    })?;
                    let path = entry.path();
                    if path.is_file() {
                        let mut perms = std::fs::metadata(&path)
                            .map_err(|e| {
                                TemplatizerError::other(format!(
                                    "Failed to get metadata: {}",
                                    e
                                ))
                            })?
                            .permissions();
                        perms.set_mode(0o755);
                        std::fs::set_permissions(&path, perms).map_err(|e| {
                            TemplatizerError::other(format!(
                                "Failed to set permissions: {}",
                                e
                            ))
                        })?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odoo::config::get_example;

    #[test]
    fn test_example_config_valid() {
        let config = get_example();
        assert!(config.validate().is_ok());
    }
}

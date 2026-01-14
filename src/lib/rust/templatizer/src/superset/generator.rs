//! Generator for Superset dashboard configurations
//!
//! This module generates Superset-compatible export files in both
//! directory and ZIP archive formats.

use crate::config::{GenerationOptions, OutputFormat, TemplateConfig};
use crate::embedded::TemplateType;
use crate::engine::TemplateEngine;
use crate::error::{Result, TemplatizerError};
use crate::superset::models::DashboardConfig;
use crate::superset::utils::sanitize_filename;
use std::io::Write;
use std::path::Path;
use tracing::{debug, info};
use zip::write::FileOptions;
use zip::ZipWriter;

/// Generator for Superset dashboards
pub struct SupersetGenerator {
    #[allow(dead_code)] // Reserved for future template-based generation
    engine: TemplateEngine,
}

impl SupersetGenerator {
    /// Create a new Superset generator
    pub fn new() -> Result<Self> {
        let engine = TemplateEngine::new(TemplateType::Superset)?;
        Ok(Self { engine })
    }

    /// Generate a dashboard to a directory
    pub fn generate(&self, config: &DashboardConfig, output_dir: &Path) -> Result<()> {
        self.generate_with_options(config, output_dir, &GenerationOptions::default())
    }

    /// Generate with specific options
    pub fn generate_with_options(
        &self,
        config: &DashboardConfig,
        output_path: &Path,
        options: &GenerationOptions,
    ) -> Result<()> {
        // Validate config
        config.validate()?;

        info!(
            "Generating Superset dashboard: {}",
            config.dashboard.title
        );

        if options.dry_run {
            info!("Dry run - would generate to: {}", output_path.display());
            return Ok(());
        }

        match options.format {
            OutputFormat::Zip => self.generate_zip(config, output_path, options),
            OutputFormat::Directory => self.generate_directory(config, output_path, options),
            OutputFormat::Stdout => {
                // For stdout, generate a preview
                let preview = self.generate_preview(config)?;
                print!("{}", preview);
                Ok(())
            }
        }
    }

    /// Generate to a directory
    fn generate_directory(
        &self,
        config: &DashboardConfig,
        output_dir: &Path,
        options: &GenerationOptions,
    ) -> Result<()> {
        // Create output directory structure
        std::fs::create_dir_all(output_dir).map_err(|e| {
            TemplatizerError::output_error(output_dir.to_path_buf(), e.to_string())
        })?;

        // Generate all YAML files
        let files = self.generate_files(config)?;

        for (path, content) in files {
            let file_path = output_dir.join(&path);

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    TemplatizerError::output_error(parent.to_path_buf(), e.to_string())
                })?;
            }

            // Check for existing file
            if file_path.exists() && !options.overwrite {
                debug!("Skipping existing file: {}", file_path.display());
                continue;
            }

            // Write file
            std::fs::write(&file_path, content).map_err(|e| {
                TemplatizerError::output_error(file_path.clone(), e.to_string())
            })?;

            if options.verbose {
                info!("Generated: {}", file_path.display());
            }
        }

        info!(
            "Successfully generated dashboard to {}",
            output_dir.display()
        );
        Ok(())
    }

    /// Generate to a ZIP archive
    fn generate_zip(
        &self,
        config: &DashboardConfig,
        output_path: &Path,
        options: &GenerationOptions,
    ) -> Result<()> {
        // Ensure path has .zip extension
        let zip_path = if output_path.extension().map_or(true, |ext| ext != "zip") {
            output_path.with_extension("zip")
        } else {
            output_path.to_path_buf()
        };

        // Check for existing file
        if zip_path.exists() && !options.overwrite {
            return Err(TemplatizerError::output_error(
                zip_path.clone(),
                "File already exists. Use --overwrite to replace.",
            ));
        }

        // Create parent directory
        if let Some(parent) = zip_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TemplatizerError::output_error(parent.to_path_buf(), e.to_string())
            })?;
        }

        // Create ZIP file
        let file = std::fs::File::create(&zip_path).map_err(|e| {
            TemplatizerError::output_error(zip_path.clone(), e.to_string())
        })?;
        let mut zip = ZipWriter::new(file);

        // Generate and add files
        let files = self.generate_files(config)?;
        let options_zip = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (path, content) in files {
            zip.start_file(&path, options_zip)?;
            zip.write_all(content.as_bytes())?;

            if options.verbose {
                info!("Added to ZIP: {}", path);
            }
        }

        zip.finish()?;

        info!("Successfully generated ZIP to {}", zip_path.display());
        Ok(())
    }

    /// Generate all files as (path, content) pairs
    fn generate_files(&self, config: &DashboardConfig) -> Result<Vec<(String, String)>> {
        let mut files = Vec::new();

        // Convert config to context for template rendering (reserved for future use)
        let _context = config.to_context()?;

        // Generate metadata.yaml
        files.push((
            "metadata.yaml".to_string(),
            self.render_metadata(config)?,
        ));

        // Generate database.yaml
        let db_path = format!("databases/{}.yaml", sanitize_filename(&config.database.name));
        files.push((db_path, self.render_database(config)?));

        // Generate datasets
        for dataset in &config.datasets {
            let dataset_path = format!(
                "datasets/{}/{}.yaml",
                sanitize_filename(&config.database.name),
                sanitize_filename(&dataset.name)
            );
            files.push((dataset_path, self.render_dataset(config, dataset)?));
        }

        // Generate charts
        for chart in &config.charts {
            let chart_path = format!("charts/{}.yaml", sanitize_filename(&chart.name));
            files.push((chart_path, self.render_chart(config, chart)?));
        }

        // Generate dashboard
        let dashboard_path = format!(
            "dashboards/{}.yaml",
            sanitize_filename(&config.dashboard.slug)
        );
        files.push((dashboard_path, self.render_dashboard(config)?));

        Ok(files)
    }

    /// Render metadata.yaml
    fn render_metadata(&self, _config: &DashboardConfig) -> Result<String> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        Ok(format!(
            r#"version: 1.0.0
type: Dashboard
timestamp: {}
"#,
            timestamp
        ))
    }

    /// Render database.yaml
    fn render_database(&self, config: &DashboardConfig) -> Result<String> {
        let db = &config.database;
        let masked_uri = config.get_safe_sqlalchemy_uri();

        Ok(format!(
            r#"database_name: {}
sqlalchemy_uri: {}
cache_timeout: null
expose_in_sqllab: {}
allow_run_async: true
allow_ctas: {}
allow_cvas: {}
allow_dml: {}
allow_csv_upload: {}
extra:
  metadata_params: {{}}
  engine_params: {{}}
  metadata_cache_timeout: {{}}
  allows_virtual_table_explore: true
uuid: {}
"#,
            db.name,
            masked_uri,
            db.options.expose_in_sqllab,
            db.options.allow_ctas,
            db.options.allow_cvas,
            db.options.allow_dml,
            db.options.allow_csv_upload,
            db.uuid
        ))
    }

    /// Render dataset.yaml
    fn render_dataset(
        &self,
        config: &DashboardConfig,
        dataset: &crate::superset::models::Dataset,
    ) -> Result<String> {
        let mut output = format!(
            r#"table_name: {}
main_dttm_col: {}
description: {}
default_endpoint: null
offset: 0
cache_timeout: null
schema: {}
sql: {}
params: null
template_params: null
filter_select_enabled: true
fetch_values_predicate: null
extra: null
uuid: {}
database_uuid: {}
"#,
            dataset.table_name,
            dataset.time_column.as_deref().unwrap_or("null"),
            if dataset.description.is_empty() {
                "null"
            } else {
                &dataset.description
            },
            dataset.schema,
            dataset.sql.as_deref().map_or("null".to_string(), |sql| {
                format!("|\n  {}", sql.replace('\n', "\n  "))
            }),
            dataset.uuid,
            config.database.uuid
        );

        // Add columns
        output.push_str("columns:\n");
        for column in &dataset.columns {
            output.push_str(&format!(
                r#"- column_name: {}
  verbose_name: {}
  is_dttm: {}
  is_active: true
  type: {}
  groupby: {}
  filterable: {}
  expression: null
  description: {}
  uuid: {}
"#,
                column.name,
                column.display_name.as_deref().unwrap_or(&column.name),
                column.is_temporal,
                column.column_type,
                column.groupable,
                column.filterable,
                if column.description.is_empty() {
                    "null"
                } else {
                    &column.description
                },
                column.uuid
            ));
        }

        // Add metrics
        output.push_str("metrics:\n");
        for metric in &dataset.metrics {
            output.push_str(&format!(
                r#"- metric_name: {}
  verbose_name: {}
  metric_type: null
  expression: {}
  description: {}
  d3format: {}
  warning_text: null
  uuid: {}
"#,
                metric.name,
                metric.display_name,
                metric.expression,
                if metric.description.is_empty() {
                    "null"
                } else {
                    &metric.description
                },
                metric.format,
                metric.uuid
            ));
        }

        Ok(output)
    }

    /// Render chart.yaml
    fn render_chart(
        &self,
        _config: &DashboardConfig,
        chart: &crate::superset::models::Chart,
    ) -> Result<String> {
        let params = serde_json::json!({
            "viz_type": crate::superset::utils::map_chart_type(&chart.chart_type),
            "metrics": chart.config.metrics,
            "groupby": chart.config.dimensions,
            "time_range": chart.config.time.as_ref().map(|t| &t.range).unwrap_or(&"No filter".to_string()),
            "granularity_sqla": chart.config.time.as_ref().map(|t| &t.column),
            "time_grain_sqla": chart.config.time.as_ref().and_then(|t| t.grain.as_ref()),
        });

        Ok(format!(
            r#"slice_name: {}
viz_type: {}
params: {}
cache_timeout: null
uuid: {}
dataset_uuid: {}
"#,
            chart.title,
            crate::superset::utils::map_chart_type(&chart.chart_type),
            serde_json::to_string_pretty(&params).unwrap_or_default(),
            chart.uuid,
            chart.dataset_uuid
        ))
    }

    /// Render dashboard.yaml
    fn render_dashboard(&self, config: &DashboardConfig) -> Result<String> {
        // Generate position_json for layout
        let position_json = self.generate_position_json(config)?;

        Ok(format!(
            r#"dashboard_title: {}
description: {}
css: {}
slug: {}
uuid: {}
position: {}
metadata:
  timed_refresh_immune_slices: []
  expanded_slices: {{}}
  refresh_frequency: {}
  default_filters: '{{}}'
  color_scheme: null
  label_colors: {{}}
  shared_label_colors: {{}}
  color_scheme_domain: []
  cross_filters_enabled: true
  native_filter_configuration: []
"#,
            config.dashboard.title,
            if config.dashboard.description.is_empty() {
                "null"
            } else {
                &config.dashboard.description
            },
            if config.dashboard.css.is_empty() {
                "null"
            } else {
                &config.dashboard.css
            },
            config.dashboard.slug,
            config.dashboard.uuid,
            position_json,
            config.dashboard.refresh_frequency
        ))
    }

    /// Generate position JSON for dashboard layout
    fn generate_position_json(&self, config: &DashboardConfig) -> Result<String> {
        let mut positions = serde_json::Map::new();

        // Root element
        positions.insert(
            "DASHBOARD_VERSION_KEY".to_string(),
            serde_json::json!("v2"),
        );
        positions.insert(
            "ROOT_ID".to_string(),
            serde_json::json!({
                "type": "ROOT",
                "id": "ROOT_ID",
                "children": ["GRID_ID"]
            }),
        );
        positions.insert(
            "GRID_ID".to_string(),
            serde_json::json!({
                "type": "GRID",
                "id": "GRID_ID",
                "children": [],
                "parents": ["ROOT_ID"]
            }),
        );
        positions.insert(
            "HEADER_ID".to_string(),
            serde_json::json!({
                "type": "HEADER",
                "id": "HEADER_ID",
                "meta": {
                    "text": config.dashboard.title
                }
            }),
        );

        // Add chart positions based on layout
        for row in &config.layout.rows {
            for component in &row.components {
                if let Some(chart_name) = &component.chart {
                    if let Some(chart) = config.charts.iter().find(|c| &c.name == chart_name) {
                        let chart_id = format!("CHART-{}", chart.uuid.to_string().replace('-', "")[..8].to_string());
                        positions.insert(
                            chart_id.clone(),
                            serde_json::json!({
                                "type": "CHART",
                                "id": chart_id,
                                "children": [],
                                "parents": ["ROOT_ID", "GRID_ID"],
                                "meta": {
                                    "width": component.width,
                                    "height": row.height,
                                    "chartId": chart.uuid.to_string(),
                                    "sliceName": chart.title
                                }
                            }),
                        );
                    }
                }
            }
        }

        let position_value = serde_json::Value::Object(positions);
        Ok(serde_json::to_string(&position_value).unwrap_or_default())
    }

    /// Generate a preview of what would be created
    fn generate_preview(&self, config: &DashboardConfig) -> Result<String> {
        let files = self.generate_files(config)?;
        let mut output = String::new();

        output.push_str(&format!(
            "Dashboard: {}\n",
            config.dashboard.title
        ));
        output.push_str(&format!("Slug: {}\n\n", config.dashboard.slug));
        output.push_str("Files that would be generated:\n");

        for (path, _) in &files {
            output.push_str(&format!("  - {}\n", path));
        }

        output.push_str(&format!(
            "\nTotal: {} files\n",
            files.len()
        ));

        Ok(output)
    }

    /// Generate to a byte vector (for upload)
    pub fn generate_zip_bytes(&self, config: &DashboardConfig) -> Result<Vec<u8>> {
        config.validate()?;

        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
            let files = self.generate_files(config)?;
            let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            for (path, content) in files {
                zip.start_file(&path, options)?;
                zip.write_all(content.as_bytes())?;
            }

            zip.finish()?;
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::superset::config::SupersetConfigBuilder;
    use crate::superset::models::{Column, Dataset};

    fn create_test_config() -> DashboardConfig {
        SupersetConfigBuilder::new("Test Dashboard", "test-dashboard")
            .database("TestDB", "localhost", 5432, "testdb")
            .credentials("user", "pass")
            .add_dataset(Dataset {
                name: "test_data".to_string(),
                table_name: "test_data".to_string(),
                columns: vec![Column {
                    name: "id".to_string(),
                    column_type: "INTEGER".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            })
            .build_unchecked()
    }

    #[test]
    fn test_generate_files() {
        // This test would require the template engine to be initialized
        // which needs embedded templates at build time
    }

    #[test]
    fn test_generate_position_json() {
        // Test position JSON generation
    }
}

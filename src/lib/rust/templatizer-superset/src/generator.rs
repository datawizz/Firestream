//! Dashboard generation logic

use crate::config::DashboardConfig;
use crate::templates::{TEMPLATES, format_time_grain, map_chart_type};
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use tera::Context;
use uuid::Uuid;
use zip::write::FileOptions;
use zip::ZipWriter;

pub struct DashboardGenerator {
    templates: &'static tera::Tera,
}

impl DashboardGenerator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            templates: &TEMPLATES,
        })
    }

    /// Generate dashboard files to a directory
    pub fn generate(&self, config: &DashboardConfig, output_dir: &Path) -> Result<()> {
        // Validate and prepare config
        let mut config = config.clone();
        config.apply_env_substitutions();
        config.initialize_uuids();
        config.validate()?;

        // Create directory structure
        fs::create_dir_all(output_dir)?;
        fs::create_dir_all(output_dir.join("databases"))?;
        fs::create_dir_all(output_dir.join("datasets"))?;
        fs::create_dir_all(output_dir.join("charts"))?;
        fs::create_dir_all(output_dir.join("dashboards"))?;

        // Generate metadata.yaml
        self.generate_metadata(&config, output_dir)?;

        // Generate database YAML
        self.generate_database(&config, output_dir)?;

        // Generate dataset YAMLs
        for dataset in &config.datasets {
            self.generate_dataset(&config, dataset, output_dir)?;
        }

        // Generate chart YAMLs
        for chart in &config.charts {
            self.generate_chart(&config, chart, output_dir)?;
        }

        // Generate dashboard YAML
        self.generate_dashboard(&config, output_dir)?;

        Ok(())
    }

    /// Generate and return ZIP file contents
    pub fn generate_zip(&self, config: &DashboardConfig) -> Result<Vec<u8>> {
        // Create temporary directory
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Generate files to temp directory
        self.generate(config, temp_path)?;

        // Create ZIP file in memory
        let mut zip_data = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut zip_data));
            let options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o644);

            // Add all files to ZIP
            self.add_directory_to_zip(&mut zip, temp_path, "", &options)?;
            
            zip.finish()?;
        }

        Ok(zip_data)
    }

    /// Generate metadata.yaml
    fn generate_metadata(&self, _config: &DashboardConfig, output_dir: &Path) -> Result<()> {
        let mut context = Context::new();
        context.insert("timestamp", &Utc::now().to_rfc3339());

        let content = self.templates.render("metadata.yaml", &context)?;
        let path = output_dir.join("metadata.yaml");
        fs::write(path, content)?;

        Ok(())
    }

    /// Generate database YAML
    fn generate_database(&self, config: &DashboardConfig, output_dir: &Path) -> Result<()> {
        let mut context = Context::new();
        let db = &config.database;

        context.insert("name", &db.name);
        context.insert("sqlalchemy_uri", &config.get_sqlalchemy_uri());
        context.insert("cache_timeout", &db.options.cache_timeout);
        context.insert("expose_in_sqllab", &db.options.expose_in_sqllab);
        context.insert("allow_run_async", &true);
        context.insert("allow_ctas", &db.options.allow_ctas);
        context.insert("allow_cvas", &db.options.allow_cvas);
        context.insert("allow_dml", &db.options.allow_dml);
        context.insert("allow_csv_upload", &db.options.allow_csv_upload);
        context.insert("sslmode", &config.database.connection.sslmode);
        context.insert("schema", "public");
        context.insert("uuid", &db.uuid.to_string());

        let content = self.templates.render("database.yaml", &context)?;
        let filename = format!("{}.yaml", sanitize_filename(&db.name));
        let path = output_dir.join("databases").join(filename);
        fs::write(path, content)?;

        Ok(())
    }

    /// Generate dataset YAML
    fn generate_dataset(&self, config: &DashboardConfig, dataset: &crate::models::Dataset, output_dir: &Path) -> Result<()> {
        let mut context = Context::new();

        context.insert("table_name", &dataset.table_name);
        context.insert("main_dttm_col", &dataset.time_column);
        context.insert("description", &dataset.description);
        context.insert("cache_timeout", &dataset.cache_timeout);
        context.insert("schema", &dataset.schema);
        context.insert("uuid", &dataset.uuid.to_string());
        context.insert("database_uuid", &config.database.uuid.to_string());
        
        // Add SQL if present for virtual datasets
        if let Some(sql) = &dataset.sql {
            context.insert("sql", sql);
        }

        // Process columns
        let columns: Vec<_> = dataset.columns.iter().map(|col| {
            json!({
                "name": col.name,
                "display_name": col.display_name.as_ref().unwrap_or(&col.name),
                "is_temporal": col.is_temporal,
                "column_type": col.column_type,
                "groupable": col.groupable,
                "filterable": col.filterable,
                "description": col.description,
                "uuid": col.uuid.to_string(),
            })
        }).collect();
        context.insert("columns", &columns);

        // Process metrics
        let metrics: Vec<_> = dataset.metrics.iter().map(|metric| {
            json!({
                "name": metric.name,
                "display_name": metric.display_name,
                "expression": metric.expression,
                "description": metric.description,
                "format": metric.format,
                "metric_type": infer_metric_type(&metric.expression),
                "uuid": metric.uuid.to_string(),
            })
        }).collect();
        context.insert("metrics", &metrics);

        let content = self.templates.render("dataset.yaml", &context)?;
        let filename = format!("{}.yaml", sanitize_filename(&dataset.name));
        let path = output_dir.join("datasets").join(filename);
        fs::write(path, content)?;

        Ok(())
    }

    /// Generate chart YAML
    fn generate_chart(&self, config: &DashboardConfig, chart: &crate::models::Chart, output_dir: &Path) -> Result<()> {
        let mut context = Context::new();

        // Find the dataset
        let dataset = config.datasets.iter()
            .find(|d| d.name == chart.dataset)
            .ok_or_else(|| anyhow::anyhow!("Dataset '{}' not found", chart.dataset))?;

        context.insert("title", &chart.title);
        context.insert("viz_type", &map_chart_type(&chart.chart_type));
        context.insert("uuid", &chart.uuid.to_string());
        context.insert("dataset_uuid", &dataset.uuid.to_string());
        context.insert("metrics", &chart.config.metrics);
        context.insert("dimensions", &chart.config.dimensions);
        context.insert("adhoc_filters", &chart.config.filters);

        // Handle time configuration
        if let Some(time_config) = &chart.config.time {
            context.insert("time_range", &time_config.range);
            if let Some(grain) = &time_config.grain {
                context.insert("time_grain", &format_time_grain(grain));
            }
        }

        // Handle chart-specific options
        let options = &chart.config.options;
        context.insert("options", options);
        
        // Extract common options
        if let Some(x_label) = options.get("x_axis_label") {
            context.insert("x_axis_label", x_label);
        }
        if let Some(y_label) = options.get("y_axis_label") {
            context.insert("y_axis_label", y_label);
        }
        if let Some(color_scheme) = options.get("color_scheme") {
            context.insert("color_scheme", color_scheme);
        }

        let content = self.templates.render("chart.yaml", &context)?;
        let filename = format!("{}.yaml", sanitize_filename(&chart.name));
        let path = output_dir.join("charts").join(filename);
        fs::write(path, content)?;

        Ok(())
    }

    /// Generate dashboard YAML
    fn generate_dashboard(&self, config: &DashboardConfig, output_dir: &Path) -> Result<()> {
        let mut context = Context::new();
        let dash = &config.dashboard;

        context.insert("title", &dash.title);
        context.insert("description", &dash.description);
        context.insert("css", &dash.css);
        context.insert("slug", &dash.slug);
        context.insert("published", &dash.published);
        context.insert("uuid", &dash.uuid.to_string());
        context.insert("refresh_frequency", &dash.refresh_frequency);

        // Handle certification
        if let Some(cert) = &dash.certification {
            context.insert("certified_by", &cert.certified_by);
            context.insert("certification_details", &cert.details);
        }

        // Generate position JSON
        let position_json = self.generate_position_json(config)?;
        context.insert("position", &position_json);

        // Generate native filters
        let native_filters = self.generate_native_filters(config)?;
        context.insert("native_filters", &native_filters);

        let content = self.templates.render("dashboard.yaml", &context)?;
        let filename = format!("{}.yaml", sanitize_filename(&dash.slug));
        let path = output_dir.join("dashboards").join(filename);
        fs::write(path, content)?;

        Ok(())
    }

    /// Generate dashboard position JSON
    fn generate_position_json(&self, config: &DashboardConfig) -> Result<serde_json::Value> {
        let mut position = json!({
            "DASHBOARD_VERSION_KEY": "v2",
            "ROOT_ID": {"type": "ROOT", "id": "ROOT_ID"},
            "GRID_ID": {
                "type": "GRID",
                "id": "GRID_ID",
                "children": []
            }
        });

        let mut row_ids = Vec::new();
        let mut component_counter = 0;

        // Process each row
        for (row_idx, row) in config.layout.rows.iter().enumerate() {
            let row_id = format!("ROW-{}", row_idx + 1);
            row_ids.push(row_id.clone());

            let mut row_children = Vec::new();

            // Process components in the row
            for component in &row.components {
                if component.component_type == "chart" {
                    if let Some(chart_name) = &component.chart {
                        // Find the chart
                        if let Some(chart) = config.charts.iter().find(|c| c.name == *chart_name) {
                            component_counter += 1;
                            let component_id = format!("CHART-{}", component_counter);
                            row_children.push(component_id.clone());

                            position[&component_id] = json!({
                                "type": "CHART",
                                "id": component_id,
                                "chartId": chart.uuid.to_string(),
                                "width": component.width,
                                "height": row.height
                            });
                        }
                    }
                }
            }

            position[&row_id] = json!({
                "type": "ROW",
                "id": row_id,
                "children": row_children
            });
        }

        position["GRID_ID"]["children"] = json!(row_ids);

        Ok(position)
    }

    /// Generate native filters configuration
    fn generate_native_filters(&self, config: &DashboardConfig) -> Result<Vec<serde_json::Value>> {
        let mut filters = Vec::new();

        for filter in &config.filters.native_filters {
            let filter_id = Uuid::new_v4();
            
            // Build target datasets/columns
            let mut targets = Vec::new();
            for target in &filter.targets {
                // Find the dataset
                if let Some(dataset) = config.datasets.iter().find(|d| d.name == target.dataset) {
                    targets.push(json!({
                        "datasetId": dataset.uuid.to_string(),
                        "column": {"name": target.column}
                    }));
                }
            }

            let filter_json = json!({
                "id": filter_id.to_string(),
                "name": filter.name,
                "filterType": filter.filter_type,
                "targets": targets,
                "defaultDataMask": {
                    "filterState": {
                        "value": filter.default_value
                    }
                },
                "cascadeParentIds": [],
                "scope": {
                    "rootPath": ["ROOT_ID"],
                    "excluded": []
                },
                "description": filter.title,
                "chartsInScope": [],
                "type": "NATIVE_FILTER"
            });

            filters.push(filter_json);
        }

        Ok(filters)
    }

    /// Add directory contents to ZIP
    fn add_directory_to_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        dir: &Path,
        prefix: &str,
        options: &FileOptions,
    ) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            
            let zip_path = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };

            if path.is_dir() {
                self.add_directory_to_zip(zip, &path, &zip_path, options)?;
            } else {
                zip.start_file(&zip_path, *options)?;
                let contents = fs::read(&path)?;
                zip.write_all(&contents)?;
            }
        }

        Ok(())
    }
}

/// Sanitize filename for filesystem
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .replace(' ', "_")
}

/// Infer metric type from expression
fn infer_metric_type(expression: &str) -> &'static str {
    let expr_upper = expression.to_uppercase();
    if expr_upper.starts_with("COUNT") {
        "count"
    } else if expr_upper.starts_with("SUM") {
        "sum"
    } else if expr_upper.starts_with("AVG") {
        "avg"
    } else if expr_upper.starts_with("MIN") {
        "min"
    } else if expr_upper.starts_with("MAX") {
        "max"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::models::*; // Already imported via super::*

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file:name"), "test_file_name");
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
    }

    #[test]
    fn test_infer_metric_type() {
        assert_eq!(infer_metric_type("COUNT(*)"), "count");
        assert_eq!(infer_metric_type("SUM(amount)"), "sum");
        assert_eq!(infer_metric_type("AVG(price)"), "avg");
        assert_eq!(infer_metric_type("custom_expression"), "");
    }
}

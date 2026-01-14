//! Configuration management and validation for Superset dashboards

use crate::config::TemplateConfig;
use crate::error::{Result, TemplatizerError};
use crate::superset::models::*;
use crate::superset::utils::validate_virtual_dataset_sql;
use std::collections::HashSet;
use uuid::Uuid;

impl DashboardConfig {
    /// Validate the configuration
    pub fn validate_config(&self) -> Result<()> {
        // Check dashboard metadata
        if self.dashboard.title.is_empty() {
            return Err(TemplatizerError::invalid_config(
                "Dashboard title cannot be empty",
            ));
        }
        if self.dashboard.slug.is_empty() {
            return Err(TemplatizerError::invalid_config(
                "Dashboard slug cannot be empty",
            ));
        }

        // Check database
        if self.database.name.is_empty() {
            return Err(TemplatizerError::invalid_config(
                "Database name cannot be empty",
            ));
        }
        if self.database.connection.database.is_empty() {
            return Err(TemplatizerError::invalid_config(
                "Database connection database name cannot be empty",
            ));
        }

        // Check datasets
        if self.datasets.is_empty() {
            return Err(TemplatizerError::invalid_config(
                "At least one dataset is required",
            ));
        }

        let dataset_names: HashSet<_> = self.datasets.iter().map(|d| &d.name).collect();
        if dataset_names.len() != self.datasets.len() {
            return Err(TemplatizerError::invalid_config(
                "Dataset names must be unique",
            ));
        }

        for dataset in &self.datasets {
            // Validate virtual datasets
            if let Some(sql) = &dataset.sql {
                // Validate SQL syntax
                validate_virtual_dataset_sql(sql).map_err(|e| {
                    TemplatizerError::invalid_config(format!(
                        "Dataset '{}' has invalid SQL: {}",
                        dataset.name, e
                    ))
                })?;

                // Virtual datasets should use name as table_name
                if dataset.is_virtual && dataset.table_name != dataset.name {
                    return Err(TemplatizerError::invalid_config(format!(
                        "Virtual dataset '{}' should use its name as table_name",
                        dataset.name
                    )));
                }
            }

            // For physical datasets, require columns
            if dataset.sql.is_none() && dataset.columns.is_empty() {
                return Err(TemplatizerError::invalid_config(format!(
                    "Dataset '{}' must have at least one column or be defined with SQL",
                    dataset.name
                )));
            }
        }

        // Check charts
        for chart in &self.charts {
            if !dataset_names.contains(&chart.dataset) {
                return Err(TemplatizerError::invalid_config(format!(
                    "Chart '{}' references unknown dataset '{}'",
                    chart.name, chart.dataset
                )));
            }
            if chart.config.metrics.is_empty() && chart.chart_type != "table" {
                return Err(TemplatizerError::invalid_config(format!(
                    "Chart '{}' must have at least one metric",
                    chart.name
                )));
            }
        }

        // Check layout
        let chart_names: HashSet<_> = self.charts.iter().map(|c| &c.name).collect();
        for row in &self.layout.rows {
            for component in &row.components {
                if component.component_type == "chart" {
                    if let Some(chart_name) = &component.chart {
                        if !chart_names.contains(chart_name) {
                            return Err(TemplatizerError::invalid_config(format!(
                                "Layout references unknown chart '{}'",
                                chart_name
                            )));
                        }
                    } else {
                        return Err(TemplatizerError::invalid_config(
                            "Chart component must specify a chart name",
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Initialize all UUIDs and relationships
    pub fn initialize_uuids(&mut self) {
        // Generate dashboard UUID if not set
        if self.dashboard.uuid == Uuid::nil() {
            self.dashboard.uuid = Uuid::new_v4();
        }

        // Generate database UUID if not set
        if self.database.uuid == Uuid::nil() {
            self.database.uuid = Uuid::new_v4();
        }

        // Set database UUID on all datasets and generate dataset UUIDs
        for dataset in &mut self.datasets {
            dataset.database_uuid = self.database.uuid;
            if dataset.uuid == Uuid::nil() {
                dataset.uuid = Uuid::new_v4();
            }

            // Generate column UUIDs
            for column in &mut dataset.columns {
                if column.uuid == Uuid::nil() {
                    column.uuid = Uuid::new_v4();
                }
            }

            // Generate metric UUIDs
            for metric in &mut dataset.metrics {
                if metric.uuid == Uuid::nil() {
                    metric.uuid = Uuid::new_v4();
                }
            }
        }

        // Set dataset UUIDs on charts and generate chart UUIDs
        for chart in &mut self.charts {
            if chart.uuid == Uuid::nil() {
                chart.uuid = Uuid::new_v4();
            }

            // Find the dataset UUID
            if let Some(dataset) = self.datasets.iter().find(|d| d.name == chart.dataset) {
                chart.dataset_uuid = dataset.uuid;
            }
        }
    }

    /// Apply environment variable substitutions
    pub fn apply_env_substitutions(&mut self) {
        self.database.connection.username =
            substitute_env(&self.database.connection.username);
        self.database.connection.password =
            substitute_env(&self.database.connection.password);
        self.database.connection.host = substitute_env(&self.database.connection.host);
        self.database.connection.database =
            substitute_env(&self.database.connection.database);
    }

    /// Get the SQLAlchemy URI for the database
    pub fn get_sqlalchemy_uri(&self) -> String {
        let conn = &self.database.connection;
        format!(
            "postgresql+psycopg2://{}:{}@{}:{}/{}",
            conn.username, conn.password, conn.host, conn.port, conn.database
        )
    }

    /// Get safe SQLAlchemy URI (without password)
    pub fn get_safe_sqlalchemy_uri(&self) -> String {
        let conn = &self.database.connection;
        format!(
            "postgresql+psycopg2://{}:***@{}:{}/{}",
            conn.username, conn.host, conn.port, conn.database
        )
    }
}

/// Substitute environment variables in a string
fn substitute_env(value: &str) -> String {
    let mut result = value.to_string();

    // Find ${VAR} patterns using a simple approach
    let mut start = 0;
    while let Some(dollar_pos) = result[start..].find("${") {
        let abs_dollar_pos = start + dollar_pos;
        if let Some(close_pos) = result[abs_dollar_pos..].find('}') {
            let abs_close_pos = abs_dollar_pos + close_pos;
            let var_name = &result[abs_dollar_pos + 2..abs_close_pos];
            if let Ok(var_value) = std::env::var(var_name) {
                let pattern = format!("${{{}}}", var_name);
                result = result.replace(&pattern, &var_value);
                // Don't advance start since we replaced the pattern
            } else {
                start = abs_close_pos + 1;
            }
        } else {
            break;
        }
    }

    result
}

impl TemplateConfig for DashboardConfig {
    fn project_name(&self) -> &str {
        &self.dashboard.slug
    }

    fn validate(&self) -> Result<()> {
        self.validate_config()
    }
}

/// Configuration builder for fluent API
pub struct SupersetConfigBuilder {
    config: DashboardConfig,
}

impl SupersetConfigBuilder {
    pub fn new(title: &str, slug: &str) -> Self {
        let config = DashboardConfig {
            dashboard: DashboardMetadata {
                title: title.to_string(),
                slug: slug.to_string(),
                ..Default::default()
            },
            database: DatabaseConfig::default(),
            datasets: Vec::new(),
            charts: Vec::new(),
            layout: DashboardLayout::default(),
            filters: FilterConfig::default(),
            access: AccessConfig::default(),
        };

        Self { config }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.config.dashboard.description = desc.to_string();
        self
    }

    pub fn database(mut self, name: &str, host: &str, port: u16, database: &str) -> Self {
        self.config.database.name = name.to_string();
        self.config.database.connection.host = host.to_string();
        self.config.database.connection.port = port;
        self.config.database.connection.database = database.to_string();
        self
    }

    pub fn credentials(mut self, username: &str, password: &str) -> Self {
        self.config.database.connection.username = username.to_string();
        self.config.database.connection.password = password.to_string();
        self
    }

    pub fn add_dataset(mut self, dataset: Dataset) -> Self {
        self.config.datasets.push(dataset);
        self
    }

    pub fn add_chart(mut self, chart: Chart) -> Self {
        self.config.charts.push(chart);
        self
    }

    pub fn add_row(mut self, height: u32, components: Vec<LayoutComponent>) -> Self {
        self.config.layout.rows.push(LayoutRow { height, components });
        self
    }

    pub fn build(mut self) -> Result<DashboardConfig> {
        self.config.apply_env_substitutions();
        self.config.initialize_uuids();
        self.config.validate_config()?;
        Ok(self.config)
    }

    pub fn build_unchecked(mut self) -> DashboardConfig {
        self.config.apply_env_substitutions();
        self.config.initialize_uuids();
        self.config
    }
}

impl crate::config::ConfigBuilder for SupersetConfigBuilder {
    type Config = DashboardConfig;

    fn new() -> Self {
        Self::new("", "")
    }

    fn project_name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.config.dashboard.title = name.clone();
        self.config.dashboard.slug = name.to_lowercase().replace(' ', "-");
        self
    }

    fn build(self) -> Result<Self::Config> {
        SupersetConfigBuilder::build(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = DashboardConfig {
            dashboard: DashboardMetadata {
                title: "Test".to_string(),
                slug: "test".to_string(),
                ..Default::default()
            },
            database: DatabaseConfig {
                name: "Test DB".to_string(),
                connection: PostgresConnection {
                    database: "test".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
            datasets: vec![Dataset {
                name: "test_data".to_string(),
                table_name: "test_data".to_string(),
                columns: vec![Column {
                    name: "id".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            charts: vec![],
            layout: DashboardLayout::default(),
            filters: FilterConfig::default(),
            access: AccessConfig::default(),
        };

        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_env_substitution() {
        std::env::set_var("TEST_VAR_SUPERSET", "test_value");
        let result = substitute_env("prefix_${TEST_VAR_SUPERSET}_suffix");
        assert_eq!(result, "prefix_test_value_suffix");
        std::env::remove_var("TEST_VAR_SUPERSET");
    }

    #[test]
    fn test_builder() {
        let config = SupersetConfigBuilder::new("Test Dashboard", "test-dashboard")
            .description("A test dashboard")
            .database("TestDB", "localhost", 5432, "testdb")
            .credentials("user", "pass")
            .add_dataset(Dataset {
                name: "test".to_string(),
                table_name: "test".to_string(),
                columns: vec![Column {
                    name: "id".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            })
            .build();

        assert!(config.is_ok());
    }
}

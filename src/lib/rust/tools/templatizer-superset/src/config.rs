//! Configuration management and validation

use crate::models::*;
use crate::utils::validate_virtual_dataset_sql;
use anyhow::{bail, Result};
use std::collections::HashSet;
use uuid::Uuid;

pub use crate::models::DashboardConfig;

impl DashboardConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Check dashboard metadata
        if self.dashboard.title.is_empty() {
            bail!("Dashboard title cannot be empty");
        }
        if self.dashboard.slug.is_empty() {
            bail!("Dashboard slug cannot be empty");
        }

        // Check database
        if self.database.name.is_empty() {
            bail!("Database name cannot be empty");
        }
        if self.database.connection.database.is_empty() {
            bail!("Database connection database name cannot be empty");
        }

        // Check datasets
        if self.datasets.is_empty() {
            bail!("At least one dataset is required");
        }

        let dataset_names: HashSet<_> = self.datasets.iter().map(|d| &d.name).collect();
        if dataset_names.len() != self.datasets.len() {
            bail!("Dataset names must be unique");
        }

        for dataset in &self.datasets {
            // Validate virtual datasets
            if let Some(sql) = &dataset.sql {
                // Validate SQL syntax
                validate_virtual_dataset_sql(sql)
                    .map_err(|e| anyhow::anyhow!("Dataset '{}' has invalid SQL: {}", dataset.name, e))?;
                
                // Virtual datasets should use name as table_name
                if dataset.is_virtual && dataset.table_name != dataset.name {
                    bail!("Virtual dataset '{}' should use its name as table_name", dataset.name);
                }
            }
            
            // For physical datasets, require columns
            if dataset.sql.is_none() && dataset.columns.is_empty() {
                bail!("Dataset '{}' must have at least one column or be defined with SQL", dataset.name);
            }
        }

        // Check charts
        for chart in &self.charts {
            if !dataset_names.contains(&chart.dataset) {
                bail!("Chart '{}' references unknown dataset '{}'", chart.name, chart.dataset);
            }
            if chart.config.metrics.is_empty() && chart.chart_type != "table" {
                bail!("Chart '{}' must have at least one metric", chart.name);
            }
        }

        // Check layout
        let chart_names: HashSet<_> = self.charts.iter().map(|c| &c.name).collect();
        for row in &self.layout.rows {
            for component in &row.components {
                if component.component_type == "chart" {
                    if let Some(chart_name) = &component.chart {
                        if !chart_names.contains(chart_name) {
                            bail!("Layout references unknown chart '{}'", chart_name);
                        }
                    } else {
                        bail!("Chart component must specify a chart name");
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
        // Replace ${VAR} patterns with environment variables
        self.database.connection.username = substitute_env(&self.database.connection.username);
        self.database.connection.password = substitute_env(&self.database.connection.password);
        self.database.connection.host = substitute_env(&self.database.connection.host);
        self.database.connection.database = substitute_env(&self.database.connection.database);
    }

    /// Get the SQLAlchemy URI for the database
    pub fn get_sqlalchemy_uri(&self) -> String {
        let conn = &self.database.connection;
        format!(
            "postgresql+psycopg2://{}:{}@{}:{}/{}",
            conn.username,
            conn.password,
            conn.host,
            conn.port,
            conn.database
        )
    }

    /// Get safe SQLAlchemy URI (without password)
    pub fn get_safe_sqlalchemy_uri(&self) -> String {
        let conn = &self.database.connection;
        format!(
            "postgresql+psycopg2://{}:***@{}:{}/{}",
            conn.username,
            conn.host,
            conn.port,
            conn.database
        )
    }
}

/// Substitute environment variables in a string
fn substitute_env(value: &str) -> String {
    let mut result = value.to_string();
    
    // Simple regex to find ${VAR} patterns
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
    
    for cap in re.captures_iter(value) {
        let var_name = &cap[1];
        if let Ok(var_value) = std::env::var(var_name) {
            result = result.replace(&cap[0], &var_value);
        }
    }
    
    result
}

/// Configuration builder for fluent API
pub struct ConfigBuilder {
    config: DashboardConfig,
}

impl ConfigBuilder {
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
        self.config.validate()?;
        Ok(self.config)
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

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_env_substitution() {
        std::env::set_var("TEST_VAR", "test_value");
        let result = substitute_env("prefix_${TEST_VAR}_suffix");
        assert_eq!(result, "prefix_test_value_suffix");
    }
}

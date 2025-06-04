//! Data models for Superset dashboard configuration

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub dashboard: DashboardMetadata,
    pub database: DatabaseConfig,
    pub datasets: Vec<Dataset>,
    pub charts: Vec<Chart>,
    pub layout: DashboardLayout,
    #[serde(default)]
    pub filters: FilterConfig,
    #[serde(default)]
    pub access: AccessConfig,
}

/// Dashboard metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetadata {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub slug: String,
    #[serde(default = "default_true")]
    pub published: bool,
    #[serde(default)]
    pub refresh_frequency: i32,
    #[serde(default)]
    pub css: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certification: Option<CertificationConfig>,
    #[serde(skip)]
    pub uuid: Uuid,
}

impl Default for DashboardMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            slug: String::new(),
            published: true,
            refresh_frequency: 0,
            css: String::new(),
            certification: None,
            uuid: Uuid::new_v4(),
        }
    }
}

/// Certification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationConfig {
    pub certified_by: String,
    pub details: String,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub connection: PostgresConnection,
    #[serde(default)]
    pub options: DatabaseOptions,
    #[serde(skip)]
    pub uuid: Uuid,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            name: "PostgreSQL".to_string(),
            connection: PostgresConnection::default(),
            options: DatabaseOptions::default(),
            uuid: Uuid::new_v4(),
        }
    }
}

/// PostgreSQL connection details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConnection {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_sslmode")]
    pub sslmode: String,
}

impl Default for PostgresConnection {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            database: String::new(),
            username: String::new(),
            password: String::new(),
            sslmode: default_sslmode(),
        }
    }
}

/// Database options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOptions {
    #[serde(default = "default_true")]
    pub allow_csv_upload: bool,
    #[serde(default = "default_true")]
    pub allow_ctas: bool,
    #[serde(default = "default_true")]
    pub allow_cvas: bool,
    #[serde(default)]
    pub allow_dml: bool,
    #[serde(default = "default_true")]
    pub expose_in_sqllab: bool,
    pub cache_timeout: Option<i32>,
}

impl Default for DatabaseOptions {
    fn default() -> Self {
        Self {
            allow_csv_upload: true,
            allow_ctas: true,
            allow_cvas: true,
            allow_dml: false,
            expose_in_sqllab: true,
            cache_timeout: None,
        }
    }
}

/// Dataset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub name: String,
    pub table_name: String,
    #[serde(default = "default_schema")]
    pub schema: String,
    #[serde(default)]
    pub description: String,
    
    // SQL support for virtual datasets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql: Option<String>,
    
    // Flag to indicate this is a virtual dataset
    #[serde(default)]
    pub is_virtual: bool,
    
    // Dataset type (physical or virtual)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub dataset_type: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_timeout: Option<i32>,
    pub columns: Vec<Column>,
    #[serde(default)]
    pub metrics: Vec<Metric>,
    #[serde(skip)]
    pub uuid: Uuid,
    #[serde(skip)]
    pub database_uuid: Uuid,
}

impl Default for Dataset {
    fn default() -> Self {
        Self {
            name: String::new(),
            table_name: String::new(),
            schema: default_schema(),
            description: String::new(),
            sql: None,
            is_virtual: false,
            dataset_type: None,
            time_column: None,
            cache_timeout: None,
            columns: Vec::new(),
            metrics: Vec::new(),
            uuid: Uuid::new_v4(),
            database_uuid: Uuid::new_v4(),
        }
    }
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_temporal: bool,
    #[serde(default)]
    pub is_primary_key: bool,
    #[serde(default = "default_true")]
    pub groupable: bool,
    #[serde(default = "default_true")]
    pub filterable: bool,
    #[serde(skip)]
    pub uuid: Uuid,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            name: String::new(),
            column_type: "VARCHAR".to_string(),
            display_name: None,
            description: String::new(),
            is_temporal: false,
            is_primary_key: false,
            groupable: true,
            filterable: true,
            uuid: Uuid::new_v4(),
        }
    }
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub expression: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_metric_format")]
    pub format: String,
    #[serde(skip)]
    pub uuid: Uuid,
}

impl Default for Metric {
    fn default() -> Self {
        Self {
            name: String::new(),
            expression: String::new(),
            display_name: String::new(),
            description: String::new(),
            format: default_metric_format(),
            uuid: Uuid::new_v4(),
        }
    }
}

/// Chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub name: String,
    pub title: String,
    #[serde(rename = "type")]
    pub chart_type: String,
    pub dataset: String,
    #[serde(default)]
    pub description: String,
    pub config: ChartConfig,
    #[serde(skip)]
    pub uuid: Uuid,
    #[serde(skip)]
    pub dataset_uuid: Uuid,
}

impl Default for Chart {
    fn default() -> Self {
        Self {
            name: String::new(),
            title: String::new(),
            chart_type: "line".to_string(),
            dataset: String::new(),
            description: String::new(),
            config: ChartConfig::default(),
            uuid: Uuid::new_v4(),
            dataset_uuid: Uuid::new_v4(),
        }
    }
}

/// Chart configuration details
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChartConfig {
    pub metrics: Vec<String>,
    #[serde(default)]
    pub dimensions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeConfig>,
    #[serde(default)]
    pub filters: Vec<ChartFilter>,
    #[serde(default)]
    pub options: serde_json::Value,
}

/// Time configuration for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    pub column: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grain: Option<String>,
    pub range: String,
}

/// Chart filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartFilter {
    pub column: String,
    pub operator: String,
    pub value: serde_json::Value,
}

/// Dashboard layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    #[serde(rename = "type", default = "default_layout_type")]
    pub layout_type: String,
    #[serde(default = "default_width")]
    pub width: u32,
    pub rows: Vec<LayoutRow>,
}

impl Default for DashboardLayout {
    fn default() -> Self {
        Self {
            layout_type: default_layout_type(),
            width: default_width(),
            rows: Vec::new(),
        }
    }
}

/// Layout row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRow {
    #[serde(default = "default_height")]
    pub height: u32,
    pub components: Vec<LayoutComponent>,
}

/// Layout component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart: Option<String>,
    pub width: u32,
}

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterConfig {
    #[serde(default)]
    pub native_filters: Vec<NativeFilter>,
}

/// Native filter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeFilter {
    pub name: String,
    pub title: String,
    #[serde(rename = "type")]
    pub filter_type: String,
    pub targets: Vec<FilterTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    #[serde(default)]
    pub config: serde_json::Value,
}

/// Filter target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterTarget {
    pub dataset: String,
    pub column: String,
}

/// Access configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccessConfig {
    #[serde(default)]
    pub owners: Vec<String>,
    #[serde(default)]
    pub roles: Vec<String>,
}

// Helper functions for serde defaults
fn default_true() -> bool {
    true
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    5432
}

fn default_sslmode() -> String {
    "require".to_string()
}

fn default_schema() -> String {
    "public".to_string()
}

fn default_metric_format() -> String {
    ",.0f".to_string()
}

fn default_layout_type() -> String {
    "grid".to_string()
}

fn default_width() -> u32 {
    12
}

fn default_height() -> u32 {
    10
}

/// Builder for creating datasets programmatically
pub struct DatasetBuilder {
    dataset: Dataset,
}

impl DatasetBuilder {
    pub fn new(name: &str, table_name: &str) -> Self {
        let mut dataset = Dataset::default();
        dataset.name = name.to_string();
        dataset.table_name = table_name.to_string();
        Self { dataset }
    }
    
    /// Create a virtual dataset from SQL
    pub fn from_sql(name: &str, sql: &str) -> Self {
        let mut dataset = Dataset::default();
        dataset.name = name.to_string();
        dataset.table_name = name.to_string(); // Virtual datasets use name as table_name
        dataset.sql = Some(sql.to_string());
        dataset.is_virtual = true;
        dataset.dataset_type = Some("virtual".to_string());
        Self { dataset }
    }
    
    /// Add SQL to existing dataset to make it virtual
    pub fn with_sql(mut self, sql: &str) -> Self {
        self.dataset.sql = Some(sql.to_string());
        self.dataset.is_virtual = true;
        self.dataset.dataset_type = Some("virtual".to_string());
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.dataset.description = desc.to_string();
        self
    }

    pub fn time_column(mut self, col: &str) -> Self {
        self.dataset.time_column = Some(col.to_string());
        self
    }

    pub fn add_column(mut self, column: Column) -> Self {
        self.dataset.columns.push(column);
        self
    }

    pub fn add_metric(mut self, metric: Metric) -> Self {
        self.dataset.metrics.push(metric);
        self
    }

    pub fn build(self) -> Dataset {
        self.dataset
    }
}

/// Builder for creating charts programmatically
pub struct ChartBuilder {
    chart: Chart,
}

impl ChartBuilder {
    pub fn new(name: &str, title: &str, chart_type: &str) -> Self {
        let mut chart = Chart::default();
        chart.name = name.to_string();
        chart.title = title.to_string();
        chart.chart_type = chart_type.to_string();
        Self { chart }
    }

    pub fn dataset(mut self, dataset: &str) -> Self {
        self.chart.dataset = dataset.to_string();
        self
    }

    pub fn add_metric(mut self, metric: &str) -> Self {
        self.chart.config.metrics.push(metric.to_string());
        self
    }

    pub fn add_dimension(mut self, dimension: &str) -> Self {
        self.chart.config.dimensions.push(dimension.to_string());
        self
    }

    pub fn time_config(mut self, column: &str, grain: Option<&str>, range: &str) -> Self {
        self.chart.config.time = Some(TimeConfig {
            column: column.to_string(),
            grain: grain.map(|s| s.to_string()),
            range: range.to_string(),
        });
        self
    }

    pub fn options(mut self, options: serde_json::Value) -> Self {
        self.chart.config.options = options;
        self
    }

    pub fn build(self) -> Chart {
        self.chart
    }
}

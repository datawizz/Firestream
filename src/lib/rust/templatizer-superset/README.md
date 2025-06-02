# Superset Dashboard Generator

An elegant Rust library for generating Apache Superset dashboards from JSON configurations. This crate provides a high-level API for creating complete Superset dashboard packages including databases, datasets, charts, and layouts from a simple JSON specification.

## Features

- **Elegant JSON-based Configuration**: Define dashboards with a clean, intuitive JSON structure
- **PostgreSQL Optimized**: Built with PostgreSQL in mind, with sensible defaults
- **Automatic Relationship Management**: UUIDs and relationships are handled automatically
- **Tera Templating**: Flexible YAML generation using Tera templates
- **Built-in ZIP Packaging**: Generate dashboard export files ready for import
- **Direct Upload**: Upload dashboards directly to Superset via REST API
- **Environment-based Configuration**: Support for environment variables in configurations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
superset-dashboard-gen = "0.1"
```

## Quick Start

```rust
use superset_dashboard_gen::{SupersetDashboardBuilder, generate_dashboard_from_json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Method 1: From JSON file
    let json = std::fs::read_to_string("dashboard.json")?;
    generate_dashboard_from_json(&json, "http://localhost:8088").await?;

    // Method 2: Using builder
    let builder = SupersetDashboardBuilder::from_json(&json)?
        .with_database_from_env();
    
    builder.upload_to_superset("http://localhost:8088").await?;

    Ok(())
}
```

## Environment Variables

```bash
# PostgreSQL connection (referenced in JSON as ${POSTGRES_USER} etc.)
export POSTGRES_USER=myuser
export POSTGRES_PASSWORD=mypassword
export POSTGRES_HOST=localhost

# Superset credentials
export SUPERSET_USERNAME=admin
export SUPERSET_PASSWORD=admin
```

## JSON Configuration Structure

The configuration JSON has the following structure:

```json
{
  "dashboard": {
    "title": "My Dashboard",
    "description": "Dashboard description",
    "slug": "my-dashboard",
    "published": true
  },
  "database": {
    "name": "Production DB",
    "connection": {
      "host": "${POSTGRES_HOST}",
      "port": 5432,
      "database": "mydb",
      "username": "${POSTGRES_USER}",
      "password": "${POSTGRES_PASSWORD}",
      "sslmode": "require"
    }
  },
  "datasets": [...],
  "charts": [...],
  "layout": {...}
}
```

## Programmatic API

### Building Dashboards

```rust
use superset_dashboard_gen::{
    ConfigBuilder, DatasetBuilder, ChartBuilder,
    Column, Metric, LayoutComponent, LayoutRow
};

let config = ConfigBuilder::new("Sales Dashboard", "sales-dashboard")
    .description("Monthly sales analytics")
    .database("PostgreSQL", "localhost", 5432, "sales_db")
    .credentials("user", "password")
    .add_dataset(
        DatasetBuilder::new("sales_data", "fact_sales")
            .description("Sales transactions")
            .time_column("order_date")
            .add_column(Column {
                name: "order_id".to_string(),
                column_type: "BIGINT".to_string(),
                display_name: Some("Order ID".to_string()),
                ..Default::default()
            })
            .add_metric(Metric {
                name: "revenue".to_string(),
                expression: "SUM(amount)".to_string(),
                display_name: "Total Revenue".to_string(),
                format: "$,.2f".to_string(),
                ..Default::default()
            })
            .build()
    )
    .add_chart(
        ChartBuilder::new("revenue_chart", "Revenue Trend", "line")
            .dataset("sales_data")
            .add_metric("revenue")
            .time_config("order_date", Some("month"), "Last year")
            .build()
    )
    .add_row(10, vec![
        LayoutComponent {
            component_type: "chart".to_string(),
            chart: Some("revenue_chart".to_string()),
            width: 12,
        }
    ])
    .build()?;
```

### Generating Files

```rust
// Generate to directory
let generator = DashboardGenerator::new()?;
generator.generate(&config, Path::new("output/"))?;

// Generate ZIP file
let zip_data = generator.generate_zip(&config)?;
std::fs::write("dashboard.zip", zip_data)?;
```

### Uploading to Superset

```rust
let mut uploader = SupersetUploader::from_env("http://localhost:8088")?;
uploader.upload_dashboard(zip_data).await?;

// Test connection
if uploader.test_connection().await? {
    println!("Connected to Superset!");
}
```

## Advanced Features

### Custom Chart Options

```rust
use serde_json::json;

let chart = ChartBuilder::new("custom_chart", "Custom Visualization", "bar")
    .dataset("my_dataset")
    .options(json!({
        "orientation": "horizontal",
        "bar_stacked": true,
        "show_legend": true,
        "y_axis_format": ",.0f",
        "color_scheme": "googleCategory20c"
    }))
    .build();
```

### Native Filters

```json
{
  "filters": {
    "native_filters": [
      {
        "name": "date_filter",
        "title": "Date Range",
        "type": "time_range",
        "targets": [
          {
            "dataset": "sales_data",
            "column": "order_date"
          }
        ],
        "default_value": "Last 30 days"
      }
    ]
  }
}
```

### Access Control

```json
{
  "access": {
    "owners": ["admin", "data_team"],
    "roles": ["Alpha", "Gamma"]
  }
}
```

## SQL Virtual Datasets

Create datasets using SQL queries instead of physical tables:

```rust
let dataset = DatasetBuilder::from_sql(
    "daily_summary",
    r#"
    SELECT 
        DATE_TRUNC('day', order_date) as date,
        COUNT(*) as order_count,
        SUM(amount) as revenue
    FROM orders
    WHERE order_date >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY 1
    "#
)
.description("Daily order summary")
.time_column("date")
.build();
```

### JSON Configuration for SQL Datasets

```json
{
  "datasets": [
    {
      "name": "customer_metrics",
      "type": "virtual",
      "sql": "SELECT c.*, COUNT(o.id) as order_count FROM customers c LEFT JOIN orders o ON c.id = o.customer_id GROUP BY c.id",
      "description": "Customer data with order counts",
      "columns": [
        {"name": "id", "type": "INTEGER"},
        {"name": "name", "type": "VARCHAR"},
        {"name": "order_count", "type": "INTEGER"}
      ]
    }
  ]
}
```

### SQL Builder Utilities

Use the built-in SQL builder for common patterns:

```rust
use superset_dashboard_gen::utils::SqlBuilder;

// Aggregation query
let sql = SqlBuilder::aggregation_query(
    "sales_data",
    vec!["region", "product"],
    vec![("SUM", "amount", "total_sales"), ("COUNT", "*", "count")],
    Some("date"),
    Some("month"),
    vec!["status = 'completed'"]
);

// Join query
let sql = SqlBuilder::join_query(
    "orders", "o",
    "customers", "c",
    ("customer_id", "id"),
    vec![("o", "*"), ("c", "name"), ("c", "email")],
    "LEFT"
);
```

## Chart Types Supported

- `line` - Line charts
- `bar` - Bar charts  
- `pie` - Pie/donut charts
- `table` - Data tables
- `big_number_total` - KPI cards
- `area` - Area charts
- `scatter` - Scatter plots
- `heatmap` - Heat maps
- `box_plot` - Box plots
- And many more...

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build --release
```

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

//! Example of using the Superset Dashboard Generator

use anyhow::Result;
use std::path::Path;
use templatizer_superset::{
    SupersetDashboardBuilder, DashboardGenerator, SupersetUploader,
    ConfigBuilder, DatasetBuilder, ChartBuilder, Column, Metric,
    LayoutComponent
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Example 1: Load from JSON file
    println!("=== Example 1: Loading dashboard from JSON file ===");
    
    let json_path = Path::new("examples/dashboard.json");
    if json_path.exists() {
        let json = std::fs::read_to_string(json_path)?;
        
        let builder = SupersetDashboardBuilder::from_json(&json)?
            .with_database_from_env();
        
        // Generate to directory
        let output_dir = Path::new("output/example1");
        builder.generate_to_directory(output_dir)?;
        println!("✓ Generated dashboard files to: {:?}", output_dir);
        
        // Generate ZIP file
        let zip_data = builder.generate_zip()?;
        std::fs::write("output/dashboard.zip", &zip_data)?;
        println!("✓ Generated ZIP file: output/dashboard.zip");
        
        // Upload to Superset (if configured)
        if std::env::var("SUPERSET_USERNAME").is_ok() {
            println!("✓ Uploading to Superset...");
            match builder.upload_to_superset("http://localhost:8088").await {
                Ok(_) => println!("✓ Dashboard uploaded successfully!"),
                Err(e) => println!("✗ Upload failed: {}", e),
            }
        }
    }

    // Example 2: Build dashboard programmatically
    println!("\n=== Example 2: Building dashboard programmatically ===");
    
    let config = ConfigBuilder::new("Programmatic Dashboard", "prog-dashboard")
        .description("Dashboard built using the Rust API")
        .database("PostgreSQL", "localhost", 5432, "analytics")
        .credentials("${POSTGRES_USER}", "${POSTGRES_PASSWORD}")
        .add_dataset(
            DatasetBuilder::new("sales", "public.sales_data")
                .description("Sales transaction data")
                .time_column("sale_date")
                .add_column(Column {
                    name: "sale_id".to_string(),
                    column_type: "BIGINT".to_string(),
                    display_name: Some("Sale ID".to_string()),
                    description: "Unique sale identifier".to_string(),
                    is_primary_key: true,
                    ..Default::default()
                })
                .add_column(Column {
                    name: "sale_date".to_string(),
                    column_type: "TIMESTAMP".to_string(),
                    display_name: Some("Sale Date".to_string()),
                    is_temporal: true,
                    ..Default::default()
                })
                .add_column(Column {
                    name: "amount".to_string(),
                    column_type: "DECIMAL".to_string(),
                    display_name: Some("Sale Amount".to_string()),
                    groupable: false,
                    ..Default::default()
                })
                .add_metric(Metric {
                    name: "total_sales".to_string(),
                    expression: "SUM(amount)".to_string(),
                    display_name: "Total Sales".to_string(),
                    description: "Sum of all sales".to_string(),
                    format: "$,.2f".to_string(),
                    ..Default::default()
                })
                .add_metric(Metric {
                    name: "sale_count".to_string(),
                    expression: "COUNT(*)".to_string(),
                    display_name: "Number of Sales".to_string(),
                    format: ",.0f".to_string(),
                    ..Default::default()
                })
                .build()
        )
        .add_chart(
            ChartBuilder::new("sales_trend", "Sales Trend", "line")
                .dataset("sales")
                .add_metric("total_sales")
                .time_config("sale_date", Some("day"), "Last 30 days")
                .options(serde_json::json!({
                    "x_axis_label": "Date",
                    "y_axis_label": "Sales ($)",
                    "color_scheme": "supersetColors"
                }))
                .build()
        )
        .add_chart(
            ChartBuilder::new("sales_kpi", "Total Sales", "big_number_total")
                .dataset("sales")
                .add_metric("total_sales")
                .time_config("sale_date", None, "Last 7 days")
                .options(serde_json::json!({
                    "compare_lag": 7,
                    "compare_suffix": "WoW"
                }))
                .build()
        )
        .add_row(8, vec![
            LayoutComponent {
                component_type: "chart".to_string(),
                chart: Some("sales_kpi".to_string()),
                width: 4,
            },
            LayoutComponent {
                component_type: "chart".to_string(),
                chart: Some("sales_trend".to_string()),
                width: 8,
            }
        ])
        .build()?;

    // Generate files
    let generator = DashboardGenerator::new()?;
    let output_dir = Path::new("output/example2");
    generator.generate(&config, output_dir)?;
    println!("✓ Generated programmatic dashboard to: {:?}", output_dir);

    // Example 3: Test Superset connection
    println!("\n=== Example 3: Testing Superset connection ===");
    
    if let Ok(mut uploader) = SupersetUploader::from_env("http://localhost:8088") {
        match uploader.test_connection().await {
            Ok(true) => {
                println!("✓ Successfully connected to Superset!");
                
                // List databases
                match uploader.list_databases().await {
                    Ok(databases) => {
                        println!("✓ Available databases:");
                        for db in databases {
                            println!("  - {}", db);
                        }
                    }
                    Err(e) => println!("✗ Failed to list databases: {}", e),
                }
            }
            Ok(false) => println!("✗ Could not connect to Superset"),
            Err(e) => println!("✗ Connection test failed: {}", e),
        }
    } else {
        println!("ℹ️  Superset credentials not configured in environment");
    }

    println!("\n✓ All examples completed!");
    Ok(())
}

//! Example showing how to use the library programmatically

use templatizer_superset::*;
use serde_json::json;
// use std::collections::HashMap; // Currently unused

/// Example: Building a complete e-commerce dashboard programmatically
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment
    dotenv::dotenv().ok();
    
    println!("Building E-Commerce Dashboard Programmatically\n");

    // Step 1: Create the configuration using the builder pattern
    let mut config = ConfigBuilder::new("E-Commerce Analytics", "ecommerce-analytics")
        .description("Comprehensive e-commerce performance dashboard")
        .database("Production PostgreSQL", "${POSTGRES_HOST}", 5432, "ecommerce_prod")
        .credentials("${POSTGRES_USER}", "${POSTGRES_PASSWORD}")
        .build()?;

    // Step 2: Add datasets programmatically
    config.datasets.push(create_orders_dataset());
    config.datasets.push(create_products_dataset());
    config.datasets.push(create_customers_dataset());

    // Step 3: Create charts
    let charts = create_all_charts();
    for chart in charts {
        config.charts.push(chart);
    }

    // Step 4: Define layout
    config.layout = create_dashboard_layout();

    // Step 5: Add filters
    config.filters = create_filters();

    // Step 6: Set access control
    config.access = AccessConfig {
        owners: vec!["admin".to_string(), "analytics_team".to_string()],
        roles: vec!["Alpha".to_string(), "Executive".to_string()],
    };

    // Step 7: Generate the dashboard
    let builder = SupersetDashboardBuilder::from_config(config)?;
    
    // Option A: Generate to directory
    let output_dir = std::path::Path::new("output/programmatic");
    builder.generate_to_directory(output_dir)?;
    println!("✓ Generated dashboard files to: {:?}", output_dir);

    // Option B: Generate ZIP
    let zip_data = builder.generate_zip()?;
    std::fs::write("programmatic_dashboard.zip", &zip_data)?;
    println!("✓ Generated ZIP file: programmatic_dashboard.zip");
    println!("  Size: {} bytes", zip_data.len());

    // Option C: Upload to Superset (if configured)
    if std::env::var("SUPERSET_USERNAME").is_ok() {
        println!("\n✓ Ready to upload to Superset");
        println!("  Run with --features upload to enable upload functionality");
    }

    Ok(())
}

fn create_orders_dataset() -> Dataset {
    DatasetBuilder::new("orders", "fact_orders")
        .description("Order transactions with detailed line items")
        .time_column("order_date")
        .add_column(Column {
            name: "order_id".to_string(),
            column_type: "UUID".to_string(),
            display_name: Some("Order ID".to_string()),
            description: "Unique order identifier".to_string(),
            is_primary_key: true,
            groupable: true,
            filterable: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "order_date".to_string(),
            column_type: "TIMESTAMP".to_string(),
            display_name: Some("Order Date".to_string()),
            description: "Date and time of order placement".to_string(),
            is_temporal: true,
            groupable: true,
            filterable: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "customer_id".to_string(),
            column_type: "UUID".to_string(),
            display_name: Some("Customer ID".to_string()),
            description: "Reference to customer".to_string(),
            groupable: true,
            filterable: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "status".to_string(),
            column_type: "VARCHAR".to_string(),
            display_name: Some("Order Status".to_string()),
            description: "Current order status".to_string(),
            groupable: true,
            filterable: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "total_amount".to_string(),
            column_type: "DECIMAL".to_string(),
            display_name: Some("Total Amount".to_string()),
            description: "Total order value".to_string(),
            groupable: false,
            filterable: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "discount_amount".to_string(),
            column_type: "DECIMAL".to_string(),
            display_name: Some("Discount".to_string()),
            description: "Total discount applied".to_string(),
            groupable: false,
            filterable: true,
            ..Default::default()
        })
        .add_metric(Metric {
            name: "revenue".to_string(),
            expression: "SUM(total_amount - COALESCE(discount_amount, 0))".to_string(),
            display_name: "Net Revenue".to_string(),
            description: "Total revenue after discounts".to_string(),
            format: "$,.2f".to_string(),
            ..Default::default()
        })
        .add_metric(Metric {
            name: "order_count".to_string(),
            expression: "COUNT(DISTINCT order_id)".to_string(),
            display_name: "Order Count".to_string(),
            description: "Number of unique orders".to_string(),
            format: ",.0f".to_string(),
            ..Default::default()
        })
        .add_metric(Metric {
            name: "avg_order_value".to_string(),
            expression: "AVG(total_amount)".to_string(),
            display_name: "AOV".to_string(),
            description: "Average order value".to_string(),
            format: "$,.2f".to_string(),
            ..Default::default()
        })
        .build()
}

fn create_products_dataset() -> Dataset {
    DatasetBuilder::new("products", "dim_products")
        .description("Product catalog with categories and pricing")
        .add_column(Column {
            name: "product_id".to_string(),
            column_type: "UUID".to_string(),
            display_name: Some("Product ID".to_string()),
            is_primary_key: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "product_name".to_string(),
            column_type: "VARCHAR".to_string(),
            display_name: Some("Product Name".to_string()),
            ..Default::default()
        })
        .add_column(Column {
            name: "category".to_string(),
            column_type: "VARCHAR".to_string(),
            display_name: Some("Category".to_string()),
            groupable: true,
            filterable: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "price".to_string(),
            column_type: "DECIMAL".to_string(),
            display_name: Some("Price".to_string()),
            groupable: false,
            ..Default::default()
        })
        .add_metric(Metric {
            name: "product_count".to_string(),
            expression: "COUNT(DISTINCT product_id)".to_string(),
            display_name: "Product Count".to_string(),
            format: ",.0f".to_string(),
            ..Default::default()
        })
        .add_metric(Metric {
            name: "avg_price".to_string(),
            expression: "AVG(price)".to_string(),
            display_name: "Average Price".to_string(),
            format: "$,.2f".to_string(),
            ..Default::default()
        })
        .build()
}

fn create_customers_dataset() -> Dataset {
    DatasetBuilder::new("customers", "dim_customers")
        .description("Customer information with segmentation")
        .add_column(Column {
            name: "customer_id".to_string(),
            column_type: "UUID".to_string(),
            display_name: Some("Customer ID".to_string()),
            is_primary_key: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "email".to_string(),
            column_type: "VARCHAR".to_string(),
            display_name: Some("Email".to_string()),
            ..Default::default()
        })
        .add_column(Column {
            name: "country".to_string(),
            column_type: "VARCHAR".to_string(),
            display_name: Some("Country".to_string()),
            groupable: true,
            filterable: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "segment".to_string(),
            column_type: "VARCHAR".to_string(),
            display_name: Some("Customer Segment".to_string()),
            groupable: true,
            filterable: true,
            ..Default::default()
        })
        .add_column(Column {
            name: "registration_date".to_string(),
            column_type: "TIMESTAMP".to_string(),
            display_name: Some("Registration Date".to_string()),
            is_temporal: true,
            ..Default::default()
        })
        .add_metric(Metric {
            name: "customer_count".to_string(),
            expression: "COUNT(DISTINCT customer_id)".to_string(),
            display_name: "Total Customers".to_string(),
            format: ",.0f".to_string(),
            ..Default::default()
        })
        .build()
}

fn create_all_charts() -> Vec<Chart> {
    vec![
        // KPI Cards
        ChartBuilder::new("revenue_kpi", "Today's Revenue", "big_number_total")
            .dataset("orders")
            .add_metric("revenue")
            .time_config("order_date", None, "today")
            .options(json!({
                "compare_lag": 1,
                "compare_suffix": "DoD",
                "show_trend_line": true,
                "header_font_size": 0.6
            }))
            .build(),
            
        ChartBuilder::new("orders_kpi", "Orders Today", "big_number_total")
            .dataset("orders")
            .add_metric("order_count")
            .time_config("order_date", None, "today")
            .options(json!({
                "compare_lag": 7,
                "compare_suffix": "WoW"
            }))
            .build(),
            
        ChartBuilder::new("aov_kpi", "Average Order Value", "big_number_total")
            .dataset("orders")
            .add_metric("avg_order_value")
            .time_config("order_date", None, "today")
            .build(),
            
        // Trend Charts
        ChartBuilder::new("revenue_trend", "Daily Revenue Trend", "line")
            .dataset("orders")
            .add_metric("revenue")
            .time_config("order_date", Some("day"), "Last 30 days")
            .options(json!({
                "x_axis_label": "Date",
                "y_axis_label": "Revenue ($)",
                "show_markers": true,
                "marker_size": 6
            }))
            .build(),
            
        ChartBuilder::new("orders_trend", "Orders by Hour", "bar")
            .dataset("orders")
            .add_metric("order_count")
            .time_config("order_date", Some("hour"), "Last 24 hours")
            .options(json!({
                "x_axis_label": "Hour",
                "y_axis_label": "Number of Orders",
                "color_scheme": "googleCategory10"
            }))
            .build(),
            
        // Distribution Charts
        ChartBuilder::new("status_pie", "Orders by Status", "pie")
            .dataset("orders")
            .add_metric("order_count")
            .add_dimension("status")
            .time_config("order_date", None, "Last 7 days")
            .options(json!({
                "donut": true,
                "show_labels": true,
                "label_type": "key_value_percent",
                "inner_radius": 30
            }))
            .build(),
            
        ChartBuilder::new("category_revenue", "Revenue by Category", "bar")
            .dataset("orders")
            .add_metric("revenue")
            .add_dimension("products.category")
            .time_config("order_date", None, "Last month")
            .options(json!({
                "orientation": "horizontal",
                "sort_by": "revenue",
                "sort_descending": true,
                "limit": 10
            }))
            .build(),
            
        // Geographic Analysis
        ChartBuilder::new("country_distribution", "Customers by Country", "treemap")
            .dataset("customers")
            .add_metric("customer_count")
            .add_dimension("country")
            .options(json!({
                "treemap_ratio": 1.618,
                "color_scheme": "d3Category20"
            }))
            .build(),
            
        // Tables
        ChartBuilder::new("top_customers", "Top Customers", "table")
            .dataset("orders")
            .add_metric("revenue")
            .add_metric("order_count")
            .add_dimension("customers.email")
            .time_config("order_date", None, "Last quarter")
            .options(json!({
                "page_size": 25,
                "include_search": true,
                "cell_bars": ["revenue"],
                "column_config": {
                    "revenue": {"d3_format": "$,.0f"},
                    "order_count": {"d3_format": ",.0f"}
                }
            }))
            .build(),
    ]
}

fn create_dashboard_layout() -> DashboardLayout {
    DashboardLayout {
        layout_type: "grid".to_string(),
        width: 12,
        rows: vec![
            // KPI Row
            LayoutRow {
                height: 8,
                components: vec![
                    LayoutComponent {
                        component_type: "chart".to_string(),
                        chart: Some("revenue_kpi".to_string()),
                        width: 4,
                    },
                    LayoutComponent {
                        component_type: "chart".to_string(),
                        chart: Some("orders_kpi".to_string()),
                        width: 4,
                    },
                    LayoutComponent {
                        component_type: "chart".to_string(),
                        chart: Some("aov_kpi".to_string()),
                        width: 4,
                    },
                ],
            },
            // Trends Row
            LayoutRow {
                height: 10,
                components: vec![
                    LayoutComponent {
                        component_type: "chart".to_string(),
                        chart: Some("revenue_trend".to_string()),
                        width: 8,
                    },
                    LayoutComponent {
                        component_type: "chart".to_string(),
                        chart: Some("status_pie".to_string()),
                        width: 4,
                    },
                ],
            },
            // Analysis Row
            LayoutRow {
                height: 10,
                components: vec![
                    LayoutComponent {
                        component_type: "chart".to_string(),
                        chart: Some("category_revenue".to_string()),
                        width: 6,
                    },
                    LayoutComponent {
                        component_type: "chart".to_string(),
                        chart: Some("orders_trend".to_string()),
                        width: 6,
                    },
                ],
            },
            // Geographic Row
            LayoutRow {
                height: 10,
                components: vec![
                    LayoutComponent {
                        component_type: "chart".to_string(),
                        chart: Some("country_distribution".to_string()),
                        width: 12,
                    },
                ],
            },
            // Table Row
            LayoutRow {
                height: 12,
                components: vec![
                    LayoutComponent {
                        component_type: "chart".to_string(),
                        chart: Some("top_customers".to_string()),
                        width: 12,
                    },
                ],
            },
        ],
    }
}

fn create_filters() -> FilterConfig {
    FilterConfig {
        native_filters: vec![
            NativeFilter {
                name: "date_range".to_string(),
                title: "Date Range".to_string(),
                filter_type: "time_range".to_string(),
                targets: vec![
                    FilterTarget {
                        dataset: "orders".to_string(),
                        column: "order_date".to_string(),
                    },
                ],
                default_value: Some(json!("Last 30 days")),
                config: json!({
                    "display_all_options": false,
                    "can_clear": true,
                }),
            },
            NativeFilter {
                name: "country".to_string(),
                title: "Country".to_string(),
                filter_type: "select".to_string(),
                targets: vec![
                    FilterTarget {
                        dataset: "customers".to_string(),
                        column: "country".to_string(),
                    },
                ],
                default_value: None,
                config: json!({
                    "multiple": true,
                    "search_enabled": true,
                    "clear_enabled": true,
                }),
            },
            NativeFilter {
                name: "category".to_string(),
                title: "Product Category".to_string(),
                filter_type: "select".to_string(),
                targets: vec![
                    FilterTarget {
                        dataset: "products".to_string(),
                        column: "category".to_string(),
                    },
                ],
                default_value: None,
                config: json!({
                    "multiple": true,
                    "search_enabled": true,
                }),
            },
        ],
    }
}

//! Example showing how to create dashboards with SQL-based virtual datasets

use anyhow::Result;
use templatizer_superset::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("=== SQL Virtual Datasets Example ===\n");

    // Create a dashboard configuration with SQL-based datasets
    let config = ConfigBuilder::new("SQL Analytics Dashboard", "sql-analytics")
        .description("Dashboard powered by SQL virtual datasets")
        .database("PostgreSQL", "${POSTGRES_HOST}", 5432, "analytics_db")
        .credentials("${POSTGRES_USER}", "${POSTGRES_PASSWORD}")
        
        // Add a simple aggregation virtual dataset
        .add_dataset(
            DatasetBuilder::from_sql(
                "daily_sales_summary",
                r#"
SELECT 
    DATE_TRUNC('day', order_date) as date,
    COUNT(DISTINCT order_id) as order_count,
    COUNT(DISTINCT customer_id) as unique_customers,
    SUM(total_amount) as revenue,
    AVG(total_amount) as avg_order_value
FROM fact_orders
WHERE order_date >= CURRENT_DATE - INTERVAL '90 days'
GROUP BY 1
ORDER BY 1 DESC
                "#
            )
            .description("Daily aggregated sales metrics for the last 90 days")
            .time_column("date")
            .add_column(Column {
                name: "date".to_string(),
                column_type: "TIMESTAMP".to_string(),
                display_name: Some("Date".to_string()),
                is_temporal: true,
                ..Default::default()
            })
            .add_column(Column {
                name: "order_count".to_string(),
                column_type: "INTEGER".to_string(),
                display_name: Some("Number of Orders".to_string()),
                groupable: false,
                ..Default::default()
            })
            .add_column(Column {
                name: "unique_customers".to_string(),
                column_type: "INTEGER".to_string(),
                display_name: Some("Unique Customers".to_string()),
                groupable: false,
                ..Default::default()
            })
            .add_column(Column {
                name: "revenue".to_string(),
                column_type: "DECIMAL".to_string(),
                display_name: Some("Revenue".to_string()),
                groupable: false,
                ..Default::default()
            })
            .add_metric(Metric {
                name: "total_revenue".to_string(),
                expression: "SUM(revenue)".to_string(),
                display_name: "Total Revenue".to_string(),
                format: "$,.2f".to_string(),
                ..Default::default()
            })
            .build()
        )
        
        // Add a complex CTE-based virtual dataset
        .add_dataset(
            DatasetBuilder::from_sql(
                "product_performance_analysis",
                r#"
WITH product_sales AS (
    SELECT 
        p.product_id,
        p.product_name,
        p.category,
        p.subcategory,
        SUM(oi.quantity) as units_sold,
        SUM(oi.quantity * oi.unit_price) as revenue,
        COUNT(DISTINCT oi.order_id) as order_count
    FROM order_items oi
    JOIN products p ON oi.product_id = p.product_id
    WHERE oi.order_date >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY 1, 2, 3, 4
),
category_totals AS (
    SELECT 
        category,
        SUM(revenue) as category_revenue
    FROM product_sales
    GROUP BY 1
)
SELECT 
    ps.*,
    ct.category_revenue,
    (ps.revenue / ct.category_revenue * 100) as category_revenue_pct,
    RANK() OVER (PARTITION BY ps.category ORDER BY ps.revenue DESC) as category_rank,
    RANK() OVER (ORDER BY ps.revenue DESC) as overall_rank
FROM product_sales ps
JOIN category_totals ct ON ps.category = ct.category
WHERE ps.revenue > 0
                "#
            )
            .description("Product performance with category analysis and rankings")
            .add_column(Column {
                name: "product_name".to_string(),
                column_type: "VARCHAR".to_string(),
                display_name: Some("Product".to_string()),
                ..Default::default()
            })
            .add_column(Column {
                name: "category".to_string(),
                column_type: "VARCHAR".to_string(),
                display_name: Some("Category".to_string()),
                ..Default::default()
            })
            .add_column(Column {
                name: "revenue".to_string(),
                column_type: "DECIMAL".to_string(),
                display_name: Some("Revenue".to_string()),
                groupable: false,
                ..Default::default()
            })
            .add_column(Column {
                name: "category_revenue_pct".to_string(),
                column_type: "DECIMAL".to_string(),
                display_name: Some("% of Category Revenue".to_string()),
                groupable: false,
                ..Default::default()
            })
            .build()
        )
        
        // Add a customer cohort analysis dataset
        .add_dataset(
            DatasetBuilder::from_sql(
                "customer_cohort_analysis",
                r#"
WITH customer_cohorts AS (
    SELECT 
        customer_id,
        DATE_TRUNC('month', first_order_date) as cohort_month,
        first_order_date
    FROM (
        SELECT 
            customer_id,
            MIN(order_date) as first_order_date
        FROM fact_orders
        GROUP BY customer_id
    ) first_orders
),
cohort_orders AS (
    SELECT 
        cc.cohort_month,
        DATE_TRUNC('month', o.order_date) as order_month,
        COUNT(DISTINCT o.customer_id) as customers,
        COUNT(o.order_id) as orders,
        SUM(o.total_amount) as revenue
    FROM customer_cohorts cc
    JOIN fact_orders o ON cc.customer_id = o.customer_id
    GROUP BY 1, 2
)
SELECT 
    cohort_month,
    order_month,
    EXTRACT(MONTH FROM AGE(order_month, cohort_month)) as months_since_first_order,
    customers,
    orders,
    revenue,
    revenue / NULLIF(customers, 0) as revenue_per_customer
FROM cohort_orders
WHERE cohort_month >= CURRENT_DATE - INTERVAL '12 months'
ORDER BY cohort_month, order_month
                "#
            )
            .description("Customer cohort retention and revenue analysis")
            .time_column("order_month")
            .build()
        )
        
        // Use SqlBuilder to create a dataset
        .add_dataset(
            DatasetBuilder::from_sql(
                "region_summary",
                &SqlBuilder::aggregation_query(
                    "fact_orders o JOIN dim_customers c ON o.customer_id = c.customer_id",
                    vec!["c.region", "c.country"],
                    vec![
                        ("COUNT", "DISTINCT o.order_id", "order_count"),
                        ("COUNT", "DISTINCT o.customer_id", "unique_customers"),
                        ("SUM", "o.total_amount", "revenue"),
                        ("AVG", "o.total_amount", "avg_order_value"),
                    ],
                    Some("o.order_date"),
                    Some("month"),
                    vec!["o.order_date >= CURRENT_DATE - INTERVAL '6 months'"],
                )
            )
            .description("Regional sales summary by month")
            .time_column("date")
            .build()
        )
        
        // Add charts that use these SQL datasets
        .add_chart(
            ChartBuilder::new("daily_revenue_trend", "Daily Revenue Trend", "line")
                .dataset("daily_sales_summary")
                .add_metric("revenue")
                .time_config("date", Some("day"), "Last 30 days")
                .options(json!({
                    "x_axis_label": "Date",
                    "y_axis_label": "Revenue ($)",
                    "show_markers": true,
                    "show_legend": false
                }))
                .build()
        )
        
        .add_chart(
            ChartBuilder::new("top_products", "Top 10 Products by Revenue", "bar")
                .dataset("product_performance_analysis")
                .add_metric("revenue")
                .add_dimension("product_name")
                .options(json!({
                    "orientation": "horizontal",
                    "row_limit": 10,
                    "sort_by_metric": true,
                    "y_axis_label": "Product",
                    "x_axis_label": "Revenue ($)"
                }))
                .build()
        )
        
        .add_chart(
            ChartBuilder::new("cohort_retention", "Customer Cohort Retention", "heatmap")
                .dataset("customer_cohort_analysis")
                .add_metric("customers")
                .add_dimension("cohort_month")
                .add_dimension("months_since_first_order")
                .options(json!({
                    "normalize_across": "x",
                    "show_values": true,
                    "color_scheme": "blue_white_yellow"
                }))
                .build()
        )
        
        // Add layout
        .add_row(10, vec![
            LayoutComponent {
                component_type: "chart".to_string(),
                chart: Some("daily_revenue_trend".to_string()),
                width: 12,
            }
        ])
        .add_row(10, vec![
            LayoutComponent {
                component_type: "chart".to_string(),
                chart: Some("top_products".to_string()),
                width: 6,
            },
            LayoutComponent {
                component_type: "chart".to_string(),
                chart: Some("cohort_retention".to_string()),
                width: 6,
            }
        ])
        
        .build()?;

    // Display the SQL datasets
    println!("Generated dashboard with {} SQL virtual datasets:", config.datasets.len());
    for dataset in &config.datasets {
        if let Some(sql) = &dataset.sql {
            println!("\n📊 Dataset: {}", dataset.name);
            println!("   Description: {}", dataset.description);
            println!("   SQL Preview:");
            let lines: Vec<&str> = sql.lines().take(5).collect();
            for line in lines {
                println!("      {}", line);
            }
            println!("      ...");
        }
    }

    // Generate the dashboard
    let builder = SupersetDashboardBuilder::from_config(config)?
        .with_database_from_env();
    
    // Generate to directory
    let output_dir = std::path::Path::new("output/sql_example");
    builder.generate_to_directory(output_dir)?;
    println!("\n✓ Generated dashboard files to: {:?}", output_dir);
    
    // Generate ZIP
    let zip_data = builder.generate_zip()?;
    std::fs::write("sql_dashboard.zip", &zip_data)?;
    println!("✓ Generated ZIP file: sql_dashboard.zip ({} bytes)", zip_data.len());

    // Example of using SqlBuilder for different query patterns
    println!("\n=== SQL Builder Examples ===\n");
    
    // Example 1: Simple aggregation
    let simple_agg = SqlBuilder::aggregation_query(
        "sales_data",
        vec!["region"],
        vec![("SUM", "amount", "total_sales"), ("COUNT", "*", "transaction_count")],
        None,
        None,
        vec!["status = 'completed'"],
    );
    println!("Simple Aggregation Query:");
    println!("{}\n", simple_agg);
    
    // Example 2: Time-based aggregation
    let time_agg = SqlBuilder::aggregation_query(
        "orders",
        vec!["product_category"],
        vec![("SUM", "revenue", "total_revenue"), ("AVG", "revenue", "avg_revenue")],
        Some("order_date"),
        Some("week"),
        vec!["order_date >= CURRENT_DATE - INTERVAL '3 months'"],
    );
    println!("Time-based Aggregation Query:");
    println!("{}\n", time_agg);
    
    // Example 3: Join query
    let join_query = SqlBuilder::join_query(
        "orders", "o",
        "customers", "c",
        ("customer_id", "id"),
        vec![
            ("o", "order_id"),
            ("o", "order_date"),
            ("o", "total_amount"),
            ("c", "customer_name"),
            ("c", "customer_segment"),
        ],
        "LEFT",
    );
    println!("Join Query:");
    println!("{}\n", join_query);

    println!("✓ SQL datasets example completed!");
    Ok(())
}

//! Integration tests for the Superset dashboard generator

use templatizer_superset::*;
use templatizer_superset::utils::SqlBuilder;
// use std::path::Path; // Currently unused
use tempfile::TempDir;

#[test]
fn test_dashboard_config_validation() {
    let json = r#"{
        "dashboard": {
            "title": "Test Dashboard",
            "slug": "test-dashboard"
        },
        "database": {
            "name": "Test DB",
            "connection": {
                "host": "localhost",
                "port": 5432,
                "database": "test_db"
            }
        },
        "datasets": [
            {
                "name": "test_dataset",
                "table_name": "test_table",
                "columns": [
                    {
                        "name": "id",
                        "type": "INTEGER"
                    }
                ]
            }
        ],
        "charts": [],
        "layout": {
            "type": "grid",
            "width": 12,
            "rows": []
        }
    }"#;

    let builder = SupersetDashboardBuilder::from_json(json).unwrap();
    assert_eq!(builder.config().dashboard.title, "Test Dashboard");
}

#[test]
fn test_dataset_builder() {
    let dataset = DatasetBuilder::new("sales", "public.sales_data")
        .description("Sales data")
        .time_column("created_at")
        .add_column(Column {
            name: "id".to_string(),
            column_type: "BIGINT".to_string(),
            ..Default::default()
        })
        .add_metric(Metric {
            name: "total".to_string(),
            expression: "SUM(amount)".to_string(),
            display_name: "Total Sales".to_string(),
            ..Default::default()
        })
        .build();

    assert_eq!(dataset.name, "sales");
    assert_eq!(dataset.table_name, "public.sales_data");
    assert_eq!(dataset.columns.len(), 1);
    assert_eq!(dataset.metrics.len(), 1);
}

#[test]
fn test_chart_builder() {
    let chart = ChartBuilder::new("test_chart", "Test Chart", "line")
        .dataset("test_dataset")
        .add_metric("revenue")
        .add_dimension("product")
        .time_config("date", Some("day"), "Last 7 days")
        .build();

    assert_eq!(chart.name, "test_chart");
    assert_eq!(chart.title, "Test Chart");
    assert_eq!(chart.chart_type, "line");
    assert_eq!(chart.config.metrics, vec!["revenue"]);
    assert_eq!(chart.config.dimensions, vec!["product"]);
}

#[test]
fn test_config_builder() {
    let config = ConfigBuilder::new("My Dashboard", "my-dashboard")
        .description("Test dashboard")
        .database("PostgreSQL", "localhost", 5432, "testdb")
        .credentials("user", "pass")
        .build();

    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.dashboard.title, "My Dashboard");
    assert_eq!(config.database.connection.host, "localhost");
}

#[test]
fn test_dashboard_generation() {
    let json = r#"{
        "dashboard": {
            "title": "Generation Test",
            "slug": "gen-test"
        },
        "database": {
            "name": "Test DB",
            "connection": {
                "host": "localhost",
                "port": 5432,
                "database": "test"
            }
        },
        "datasets": [
            {
                "name": "test_data",
                "table_name": "test_table",
                "columns": [
                    {
                        "name": "id",
                        "type": "INTEGER",
                        "is_primary_key": true
                    },
                    {
                        "name": "created_at",
                        "type": "TIMESTAMP",
                        "is_temporal": true
                    },
                    {
                        "name": "amount",
                        "type": "DECIMAL"
                    }
                ],
                "metrics": [
                    {
                        "name": "total",
                        "expression": "SUM(amount)",
                        "display_name": "Total Amount"
                    }
                ]
            }
        ],
        "charts": [
            {
                "name": "test_chart",
                "title": "Test Chart",
                "type": "line",
                "dataset": "test_data",
                "config": {
                    "metrics": ["total"],
                    "time": {
                        "column": "created_at",
                        "grain": "day",
                        "range": "Last 7 days"
                    }
                }
            }
        ],
        "layout": {
            "rows": [
                {
                    "height": 10,
                    "components": [
                        {
                            "type": "chart",
                            "chart": "test_chart",
                            "width": 12
                        }
                    ]
                }
            ]
        }
    }"#;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();

    let builder = SupersetDashboardBuilder::from_json(json).unwrap();
    builder.generate_to_directory(output_path).unwrap();

    // Check that files were created
    assert!(output_path.join("metadata.yaml").exists());
    assert!(output_path.join("databases").exists());
    assert!(output_path.join("datasets").exists());
    assert!(output_path.join("charts").exists());
    assert!(output_path.join("dashboards").exists());

    // Check specific files
    assert!(output_path.join("databases/Test_DB.yaml").exists());
    assert!(output_path.join("datasets/test_data.yaml").exists());
    assert!(output_path.join("charts/test_chart.yaml").exists());
    assert!(output_path.join("dashboards/gen-test.yaml").exists());
}

#[test]
fn test_zip_generation() {
    let json = r#"{
        "dashboard": {
            "title": "ZIP Test",
            "slug": "zip-test"
        },
        "database": {
            "name": "Test DB",
            "connection": {
                "database": "test"
            }
        },
        "datasets": [
            {
                "name": "data",
                "table_name": "table",
                "columns": [{"name": "id", "type": "INTEGER"}]
            }
        ],
        "charts": [],
        "layout": {"rows": []}
    }"#;

    let builder = SupersetDashboardBuilder::from_json(json).unwrap();
    let zip_data = builder.generate_zip().unwrap();

    // Verify it's a valid ZIP
    assert!(zip_data.len() > 0);
    assert_eq!(&zip_data[0..2], b"PK"); // ZIP magic number
}

#[test]
fn test_environment_substitution() {
    std::env::set_var("TEST_DB_HOST", "production.example.com");
    std::env::set_var("TEST_DB_USER", "prod_user");

    let json = r#"{
        "dashboard": {
            "title": "Env Test",
            "slug": "env-test"
        },
        "database": {
            "name": "Test DB",
            "connection": {
                "host": "${TEST_DB_HOST}",
                "database": "test",
                "username": "${TEST_DB_USER}"
            }
        },
        "datasets": [
            {
                "name": "data",
                "table_name": "table",
                "columns": [{"name": "id", "type": "INTEGER"}]
            }
        ],
        "charts": [],
        "layout": {"rows": []}
    }"#;

    let builder = SupersetDashboardBuilder::from_json(json).unwrap();
    let _config = builder.config();
    
    // The substitution happens during generation, so we need to generate
    let temp_dir = TempDir::new().unwrap();
    builder.generate_to_directory(temp_dir.path()).unwrap();

    // Clean up
    std::env::remove_var("TEST_DB_HOST");
    std::env::remove_var("TEST_DB_USER");
}

#[test]
fn test_validation_errors() {
    // Missing dashboard title
    let json = r#"{
        "dashboard": {
            "title": "",
            "slug": "test"
        },
        "database": {
            "name": "DB",
            "connection": {"database": "test"}
        },
        "datasets": [],
        "charts": [],
        "layout": {"rows": []}
    }"#;

    let builder = SupersetDashboardBuilder::from_json(json).unwrap();
    let temp_dir = TempDir::new().unwrap();
    let result = builder.generate_to_directory(temp_dir.path());
    assert!(result.is_err());

    // Chart referencing non-existent dataset
    let json = r#"{
        "dashboard": {
            "title": "Test",
            "slug": "test"
        },
        "database": {
            "name": "DB",
            "connection": {"database": "test"}
        },
        "datasets": [
            {
                "name": "data",
                "table_name": "table",
                "columns": [{"name": "id", "type": "INTEGER"}]
            }
        ],
        "charts": [
            {
                "name": "chart",
                "title": "Chart",
                "type": "line",
                "dataset": "non_existent",
                "config": {"metrics": ["count"]}
            }
        ],
        "layout": {"rows": []}
    }"#;

    let builder = SupersetDashboardBuilder::from_json(json).unwrap();
    let result = builder.generate_to_directory(temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_complex_dashboard() {
    let json = std::fs::read_to_string("examples/dashboard.json");
    if let Ok(json) = json {
        let builder = SupersetDashboardBuilder::from_json(&json).unwrap();
        let temp_dir = TempDir::new().unwrap();
        
        // Should successfully generate
        builder.generate_to_directory(temp_dir.path()).unwrap();
        
        // Should successfully create ZIP
        let zip_data = builder.generate_zip().unwrap();
        assert!(zip_data.len() > 1000); // Reasonable size for a complex dashboard
    }
}

#[test]
fn test_sql_virtual_dataset() {
    let dataset = DatasetBuilder::from_sql(
        "test_view",
        "SELECT id, name, created_at FROM users WHERE active = true"
    )
    .description("Active users view")
    .time_column("created_at")
    .build();
    
    assert_eq!(dataset.name, "test_view");
    assert_eq!(dataset.table_name, "test_view");
    assert!(dataset.is_virtual);
    assert!(dataset.sql.is_some());
    assert_eq!(dataset.dataset_type, Some("virtual".to_string()));
}

#[test]
fn test_sql_dataset_in_config() {
    let config = ConfigBuilder::new("SQL Test", "sql-test")
        .database("PostgreSQL", "localhost", 5432, "test_db")
        .credentials("user", "pass")
        .add_dataset(
            DatasetBuilder::from_sql(
                "monthly_summary",
                r#"SELECT 
                    DATE_TRUNC('month', created_at) as month,
                    COUNT(*) as count,
                    SUM(amount) as total
                   FROM transactions
                   GROUP BY 1"#
            )
            .time_column("month")
            .build()
        )
        .add_chart(
            ChartBuilder::new("summary_chart", "Monthly Summary", "line")
                .dataset("monthly_summary")
                .add_metric("total")
                .time_config("month", Some("month"), "Last year")
                .build()
        )
        .build();
    
    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.datasets.len(), 1);
    assert!(config.datasets[0].sql.is_some());
}

#[test]
fn test_sql_builder_utilities() {
    
    // Test aggregation query
    let sql = SqlBuilder::aggregation_query(
        "orders",
        vec!["status"],
        vec![("COUNT", "*", "order_count"), ("SUM", "amount", "total_amount")],
        Some("created_at"),
        Some("day"),
        vec!["status != 'cancelled'"],
    );
    
    assert!(sql.contains("SELECT"));
    assert!(sql.contains("DATE_TRUNC('day', created_at) as date"));
    assert!(sql.contains("COUNT(*) as order_count"));
    assert!(sql.contains("WHERE"));
    assert!(sql.contains("GROUP BY"));
    
    // Test join query
    let join_sql = SqlBuilder::join_query(
        "orders", "o",
        "users", "u",
        ("user_id", "id"),
        vec![("o", "id"), ("o", "amount"), ("u", "name")],
        "INNER",
    );
    
    assert!(join_sql.contains("SELECT"));
    assert!(join_sql.contains("o.id"));
    assert!(join_sql.contains("INNER JOIN users u"));
    assert!(join_sql.contains("ON o.user_id = u.id"));
}

#[test]
fn test_virtual_dataset_validation() {
    // Invalid SQL should fail validation
    let json = r#"{
        "dashboard": {
            "title": "Test",
            "slug": "test"
        },
        "database": {
            "name": "DB",
            "connection": {"database": "test"}
        },
        "datasets": [
            {
                "name": "bad_sql",
                "type": "virtual",
                "sql": "DROP TABLE users; SELECT * FROM users",
                "columns": []
            }
        ],
        "charts": [],
        "layout": {"rows": []}
    }"#;
    
    let builder = SupersetDashboardBuilder::from_json(json).unwrap();
    let temp_dir = TempDir::new().unwrap();
    let result = builder.generate_to_directory(temp_dir.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid SQL"));
}

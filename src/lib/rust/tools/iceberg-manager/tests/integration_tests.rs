use iceberg_manager::{CatalogConfig, TableManager, Schema, NestedField, Type, PrimitiveType};
use std::collections::HashMap;

#[tokio::test]
async fn test_create_namespace_and_table() {
    // Use memory catalog for testing
    let catalog_config = CatalogConfig::Memory {
        warehouse: "/tmp/test-iceberg-warehouse".to_string(),
    };
    
    let manager = TableManager::new(catalog_config)
        .await
        .expect("Failed to create table manager");
    
    // Create namespace
    manager
        .create_namespace("test_namespace", HashMap::new())
        .await
        .expect("Failed to create namespace");
    
    // Verify namespace exists
    let namespaces = manager.list_namespaces().await.unwrap();
    assert!(namespaces.contains(&"test_namespace".to_string()));
    
    // Create schema using the official API
    let schema = Schema::builder()
        .with_schema_id(1)
        .with_fields(vec![
            NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)).into(),
            NestedField::required(2, "name", Type::Primitive(PrimitiveType::String)).into(),
        ])
        .build()
        .unwrap();
    
    // Create table
    manager
        .create_table("test_namespace", "test_table", schema, None)
        .await
        .expect("Failed to create table");
    
    // Verify table exists
    let tables = manager.list_tables("test_namespace").await.unwrap();
    assert!(tables.contains(&"test_table".to_string()));
}

#[tokio::test]
async fn test_sql_query_integration() {
    let catalog_config = CatalogConfig::Memory {
        warehouse: "/tmp/test-sql-warehouse".to_string(),
    };
    
    let manager = TableManager::new(catalog_config)
        .await
        .expect("Failed to create table manager");
    
    // Create namespace and table
    manager
        .create_namespace("sql_test", HashMap::new())
        .await
        .unwrap();
    
    // Create schema using the official API
    let schema = Schema::builder()
        .with_schema_id(1)
        .with_fields(vec![
            NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)).into(),
            NestedField::required(2, "value", Type::Primitive(PrimitiveType::Double)).into(),
        ])
        .build()
        .unwrap();
    
    manager
        .create_table("sql_test", "metrics", schema, None)
        .await
        .unwrap();
    
    // Register table with DataFusion
    manager
        .register_table("metrics", "sql_test")
        .await
        .unwrap();
    
    // Test SQL query (table will be empty but query should work)
    let df = manager
        .sql("SELECT COUNT(*) as count FROM metrics")
        .await
        .expect("SQL query failed");
    
    // Verify we can collect results
    let batches = df.collect().await.expect("Failed to collect results");
    assert!(!batches.is_empty());
}

#[tokio::test]
async fn test_table_schema_inspection() {
    let catalog_config = CatalogConfig::Memory {
        warehouse: "/tmp/test-schema-warehouse".to_string(),
    };
    
    let manager = TableManager::new(catalog_config)
        .await
        .unwrap();
    
    manager
        .create_namespace("schema_test", HashMap::new())
        .await
        .unwrap();
    
    // Create schema using the official API
    let schema = Schema::builder()
        .with_schema_id(1)
        .with_fields(vec![
            NestedField::required(1, "user_id", Type::Primitive(PrimitiveType::Long))
                .with_doc("User identifier")
                .into(),
            NestedField::optional(2, "email", Type::Primitive(PrimitiveType::String))
                .with_doc("User email address")
                .into(),
            NestedField::required(3, "created_at", Type::Primitive(PrimitiveType::Timestamp))
                .into(),
        ])
        .build()
        .unwrap();
    
    manager
        .create_table("schema_test", "users", schema, None)
        .await
        .unwrap();
    
    // Get and verify schema
    let table_schema = manager
        .get_table_schema("schema_test", "users")
        .await
        .unwrap();
    
    assert_eq!(table_schema.fields.len(), 3);
    
    let user_id_field = &table_schema.fields[0];
    assert_eq!(user_id_field.name, "user_id");
    assert_eq!(user_id_field.required, true);
    assert_eq!(user_id_field.data_type, "long");
    assert_eq!(user_id_field.doc, Some("User identifier".to_string()));
    
    let email_field = &table_schema.fields[1];
    assert_eq!(email_field.name, "email");
    assert_eq!(email_field.required, false);
    assert_eq!(email_field.data_type, "string");
}

#[tokio::test]
async fn test_table_stats() {
    let catalog_config = CatalogConfig::Memory {
        warehouse: "/tmp/test-stats-warehouse".to_string(),
    };
    
    let manager = TableManager::new(catalog_config)
        .await
        .unwrap();
    
    manager
        .create_namespace("stats_test", HashMap::new())
        .await
        .unwrap();
    
    let schema = Schema::builder()
        .with_schema_id(1)
        .with_fields(vec![
            NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)).into(),
        ])
        .build()
        .unwrap();
    
    manager
        .create_table("stats_test", "test_table", schema, None)
        .await
        .unwrap();
    
    // Get table statistics
    let stats = manager
        .get_table_stats("stats_test", "test_table")
        .await
        .unwrap();
    
    // Verify basic stats
    assert!(stats.location.contains("test_table"));
    assert_eq!(stats.current_snapshot_id, None); // No snapshots yet
    assert_eq!(stats.snapshot_count, 0);
}

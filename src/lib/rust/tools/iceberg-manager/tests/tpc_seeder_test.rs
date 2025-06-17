//! Integration tests for TPC data seeding functionality

use iceberg_manager::{CatalogConfig, TableManager, TpcSeeder, TpcSeederConfig, TpcBenchmark};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

async fn setup_test_environment() -> (Arc<TableManager>, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let warehouse_path = temp_dir.path().to_str().unwrap().to_string();
    
    let catalog_config = CatalogConfig::Memory { warehouse: warehouse_path };
    let table_manager = Arc::new(TableManager::new(catalog_config).await.unwrap());
    
    (table_manager, temp_dir)
}

#[tokio::test]
async fn test_tpch_seed_small_scale() {
    let (table_manager, _temp_dir) = setup_test_environment().await;
    
    // Create namespace
    table_manager.create_namespace("tpch_test", HashMap::new()).await.unwrap();
    
    // Create seeder with small batch size for testing
    let config = TpcSeederConfig {
        batch_size: 1000,
        show_progress: false,
    };
    let seeder = TpcSeeder::new(table_manager.clone(), config).unwrap();
    
    // Seed TPC-H data at smallest scale factor
    seeder.seed_tpch("tpch_test", 0.01).await.unwrap();
    
    // Verify tables were created
    let tables = table_manager.list_tables("tpch_test").await.unwrap();
    assert_eq!(tables.len(), 8);
    
    // Verify specific TPC-H tables exist
    assert!(tables.contains(&"customer".to_string()));
    assert!(tables.contains(&"lineitem".to_string()));
    assert!(tables.contains(&"nation".to_string()));
    assert!(tables.contains(&"orders".to_string()));
    assert!(tables.contains(&"part".to_string()));
    assert!(tables.contains(&"partsupp".to_string()));
    assert!(tables.contains(&"region".to_string()));
    assert!(tables.contains(&"supplier".to_string()));
    
    // Register and query a table to verify data
    table_manager.register_table("nation", "tpch_test").await.unwrap();
    let df = table_manager.sql("SELECT COUNT(*) as cnt FROM nation").await.unwrap();
    let batches = df.collect().await.unwrap();
    
    // Nation table should have 25 rows in TPC-H
    let count = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<arrow::array::Int64Array>()
        .unwrap()
        .value(0);
    assert_eq!(count, 25);
}

#[tokio::test]
async fn test_tpch_preview() {
    let (table_manager, _temp_dir) = setup_test_environment().await;
    
    let config = TpcSeederConfig {
        batch_size: 1000,
        show_progress: false,
    };
    let seeder = TpcSeeder::new(table_manager, config).unwrap();
    
    // Preview TPC-H migration
    let table_infos = seeder.preview_migration(TpcBenchmark::TpcH, 0.01).await.unwrap();
    
    // Should have 8 tables
    assert_eq!(table_infos.len(), 8);
    
    // Verify all tables have rows and columns
    for info in &table_infos {
        assert!(info.row_count > 0);
        assert!(info.column_count > 0);
    }
    
    // Check specific tables
    let nation_info = table_infos.iter().find(|t| t.name == "nation").unwrap();
    assert_eq!(nation_info.row_count, 25); // Nation always has 25 rows
    
    let region_info = table_infos.iter().find(|t| t.name == "region").unwrap();
    assert_eq!(region_info.row_count, 5); // Region always has 5 rows
}

#[tokio::test]
async fn test_empty_namespace_handling() {
    let (table_manager, _temp_dir) = setup_test_environment().await;
    
    let config = TpcSeederConfig {
        batch_size: 1000,
        show_progress: false,
    };
    let seeder = TpcSeeder::new(table_manager.clone(), config).unwrap();
    
    // Try to seed without creating namespace first
    let result = seeder.seed_tpch("nonexistent", 0.01).await;
    
    // Should fail because namespace doesn't exist
    assert!(result.is_err());
}

#[tokio::test]
async fn test_schema_conversion() {
    let (table_manager, _temp_dir) = setup_test_environment().await;
    
    // Create namespace
    table_manager.create_namespace("schema_test", HashMap::new()).await.unwrap();
    
    let config = TpcSeederConfig {
        batch_size: 1000,
        show_progress: false,
    };
    let seeder = TpcSeeder::new(table_manager.clone(), config).unwrap();
    
    // Seed data
    seeder.seed_tpch("schema_test", 0.01).await.unwrap();
    
    // Check schema of lineitem table (most complex in TPC-H)
    let schema = table_manager.get_table_schema("schema_test", "lineitem").await.unwrap();
    
    // Lineitem should have 16 columns in TPC-H
    assert_eq!(schema.fields.len(), 16);
    
    // Verify some key fields
    let orderkey_field = schema.fields.iter().find(|f| f.name == "l_orderkey").unwrap();
    assert_eq!(orderkey_field.data_type, "long");
    
    let quantity_field = schema.fields.iter().find(|f| f.name == "l_quantity").unwrap();
    assert!(quantity_field.data_type.contains("decimal"));
}

#[tokio::test]
async fn test_batch_processing() {
    let (table_manager, _temp_dir) = setup_test_environment().await;
    
    // Create namespace
    table_manager.create_namespace("batch_test", HashMap::new()).await.unwrap();
    
    // Use very small batch size to test batching logic
    let config = TpcSeederConfig {
        batch_size: 10,  // Very small to force multiple batches
        show_progress: false,
    };
    let seeder = TpcSeeder::new(table_manager.clone(), config).unwrap();
    
    // Seed data
    seeder.seed_tpch("batch_test", 0.01).await.unwrap();
    
    // Query a table that should have been processed in batches
    table_manager.register_table("customer", "batch_test").await.unwrap();
    let df = table_manager.sql("SELECT COUNT(*) as cnt FROM customer").await.unwrap();
    let batches = df.collect().await.unwrap();
    
    // Verify all data was loaded despite small batch size
    let count = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<arrow::array::Int64Array>()
        .unwrap()
        .value(0);
    assert!(count > 10); // Should have more rows than batch size
}

#[tokio::test]
async fn test_tpcds_preview() {
    let (table_manager, _temp_dir) = setup_test_environment().await;
    
    let config = TpcSeederConfig {
        batch_size: 1000,
        show_progress: false,
    };
    let seeder = TpcSeeder::new(table_manager, config).unwrap();
    
    // Preview TPC-DS migration
    let table_infos = seeder.preview_migration(TpcBenchmark::TpcDs, 0.01).await.unwrap();
    
    // TPC-DS has many more tables than TPC-H
    assert!(table_infos.len() > 8);
    
    // Verify all tables have rows and columns
    for info in &table_infos {
        assert!(info.column_count > 0);
        // Some TPC-DS tables might be empty at very small scale factors
    }
}

#[tokio::test]
async fn test_concurrent_namespace_seeding() {
    let (table_manager, _temp_dir) = setup_test_environment().await;
    
    // Create two namespaces
    table_manager.create_namespace("namespace1", HashMap::new()).await.unwrap();
    table_manager.create_namespace("namespace2", HashMap::new()).await.unwrap();
    
    let config = TpcSeederConfig {
        batch_size: 1000,
        show_progress: false,
    };
    
    // Seed data to first namespace
    let seeder1 = TpcSeeder::new(table_manager.clone(), config.clone()).unwrap();
    seeder1.seed_tpch("namespace1", 0.01).await.unwrap();
    
    // Seed data to second namespace
    let seeder2 = TpcSeeder::new(table_manager.clone(), config).unwrap();
    seeder2.seed_tpch("namespace2", 0.01).await.unwrap();
    
    // Verify both namespaces have their own tables
    let tables1 = table_manager.list_tables("namespace1").await.unwrap();
    let tables2 = table_manager.list_tables("namespace2").await.unwrap();
    
    assert_eq!(tables1.len(), 8);
    assert_eq!(tables2.len(), 8);
}

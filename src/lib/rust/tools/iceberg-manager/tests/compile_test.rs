use iceberg_manager::{TableManager, CatalogConfig};

#[tokio::test]
async fn test_basic_compilation() {
    // This test just ensures basic types compile
    let catalog_config = CatalogConfig::Memory {
        warehouse: "/tmp/test-compile-warehouse".to_string(),
    };
    
    let _manager = TableManager::new(catalog_config).await.unwrap();
}

// Note: When importing your own crate in examples, use the crate name directly
use firestream_tui::models::{StorageConfig, StorageType, StorageCredentials};
use firestream_tui::services::IcebergService;
use iceberg_manager::{Schema, NestedField, Type, PrimitiveType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Iceberg service
    let iceberg_service = IcebergService::new();
    
    // Initialize default catalogs (creates local filesystem catalog)
    iceberg_service.init_default_catalogs().await?;
    
    // Example 1: Working with local catalog
    println!("=== Local Catalog Example ===");
    
    // Get the table manager for local catalog
    let local_manager = iceberg_service.get_manager("local").await?;
    
    // Create a namespace
    local_manager.create_namespace("example", Default::default()).await?;
    println!("Created namespace: example");
    
    // Create a simple table schema
    let schema = Schema::builder()
        .with_schema_id(1)
        .with_fields(vec![
            NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)).into(),
            NestedField::required(2, "name", Type::Primitive(PrimitiveType::String)).into(),
            NestedField::optional(3, "email", Type::Primitive(PrimitiveType::String)).into(),
            NestedField::required(4, "created_at", Type::Primitive(PrimitiveType::Timestamp)).into(),
        ])
        .build()?;
    
    // Create a table
    local_manager.create_table("example", "users", schema, None).await?;
    println!("Created table: example.users");
    
    // List tables in namespace
    let tables = local_manager.list_tables("example").await?;
    println!("Tables in example namespace: {:?}", tables);
    
    // Get table schema
    let table_schema = local_manager.get_table_schema("example", "users").await?;
    println!("\nTable schema:");
    println!("{}", table_schema.display());
    
    // Register table for SQL queries
    local_manager.register_table("users", "example").await?;
    
    // Execute a SQL query (note: table needs data to return results)
    let df = local_manager.sql("SELECT * FROM users LIMIT 10").await?;
    println!("\nQuery results:");
    df.show().await?;
    
    // Example 2: Creating an S3 catalog (requires REST catalog server)
    println!("\n=== S3 Catalog Example (Configuration Only) ===");
    
    let s3_config = StorageConfig {
        storage_type: StorageType::S3,
        credentials: StorageCredentials::S3 {
            access_key_id: "your-access-key".to_string(),
            secret_access_key: "your-secret-key".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
        },
    };
    
    // This would create an S3-backed catalog if REST catalog server is available
    // let s3_catalog = iceberg_service.create_catalog("s3_prod", s3_config).await?;
    println!("S3 catalog configuration created (REST catalog server required for actual use)");
    
    // Example 3: List all catalogs
    println!("\n=== All Catalogs ===");
    let catalogs = iceberg_service.list_catalogs().await?;
    for catalog in catalogs {
        println!("Catalog: {} (type: {:?})", catalog.name, catalog.catalog_type);
        println!("  Warehouse: {}", catalog.warehouse);
        println!("  Namespaces: {:?}", catalog.namespaces);
    }
    
    Ok(())
}

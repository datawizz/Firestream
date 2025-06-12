use anyhow::Result;
use iceberg_manager::{
    TableManager, CatalogConfig, Schema, NestedField, Type, PrimitiveType
};
use std::collections::HashMap;

/// This example demonstrates basic operations with the Iceberg manager
#[tokio::main]
async fn main() -> Result<()> {
    println!("Iceberg Manager Example");
    println!("=======================\n");
    
    // Create catalog configuration for memory catalog
    let config = CatalogConfig::Memory {
        warehouse: "/tmp/iceberg-warehouse".to_string(),
    };
    
    // Initialize the table manager
    println!("Initializing memory catalog...");
    let manager = TableManager::new(config).await?;
    
    // Create a namespace
    println!("\nCreating namespace 'example.data'...");
    match manager.create_namespace("example", HashMap::new()).await {
        Ok(_) => println!("✓ Created namespace: example"),
        Err(e) => println!("! Namespace might already exist: {}", e),
    }
    
    match manager.create_namespace("example.data", HashMap::new()).await {
        Ok(_) => println!("✓ Created namespace: example.data"),
        Err(e) => println!("! Namespace might already exist: {}", e),
    }
    
    // List namespaces
    println!("\nListing all namespaces:");
    let namespaces = manager.list_namespaces().await?;
    for ns in namespaces {
        println!("  - {}", ns);
    }
    
    // Create a table
    println!("\nCreating a table 'users' in 'example.data' namespace...");
    
    // Build schema using the official API
    let schema = Schema::builder()
        .with_schema_id(1)
        .with_fields(vec![
            NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)).into(),
            NestedField::required(2, "username", Type::Primitive(PrimitiveType::String)).into(),
            NestedField::optional(3, "email", Type::Primitive(PrimitiveType::String)).into(),
            NestedField::required(4, "created_at", Type::Primitive(PrimitiveType::Timestamptz)).into(),
        ])
        .build()?;
    
    match manager.create_table(
        "example.data",
        "users",
        schema,
        None,
    ).await {
        Ok(_) => println!("✓ Created table: users"),
        Err(e) => println!("! Table creation failed or table already exists: {}", e),
    }
    
    // List tables in namespace
    println!("\nListing tables in 'example.data':");
    let tables = manager.list_tables("example.data").await?;
    for table in &tables {
        println!("  - {}", table);
    }
    
    // Get table schema
    if tables.contains(&"users".to_string()) {
        println!("\nTable schema for 'users':");
        let schema = manager.get_table_schema("example.data", "users").await?;
        println!("{}", schema.display());
        
        // Get table statistics
        println!("\nTable statistics for 'users':");
        let stats = manager.get_table_stats("example.data", "users").await?;
        println!("{}", stats.display());
    }
    
    println!("\nExample completed successfully!");
    println!("\nYou can now:");
    println!("  - Use the CLI to manage tables");
    println!("  - Use Apache Spark or PyIceberg to write data to the tables");
    println!("  - Use this manager to read and analyze the data with DataFusion SQL");
    
    Ok(())
}

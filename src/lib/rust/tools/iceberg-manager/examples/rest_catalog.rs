//! Example: Connect to a REST catalog and query tables

use anyhow::Result;
use iceberg_manager::{CatalogConfig, TableManager};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // REST catalog configuration
    // The REST catalog will use authentication if provided in properties
    let props = HashMap::new();
    // Add any authentication headers or tokens here if needed
    // props.insert("header.Authorization".to_string(), "Bearer <token>".to_string());
    
    // Connect to REST catalog
    let catalog_config = CatalogConfig::Rest {
        uri: "http://localhost:8181".to_string(),
        warehouse: "s3://my-iceberg-data/warehouse".to_string(),
        props,
    };
    
    // Create table manager
    let manager = TableManager::new(catalog_config).await?;
    
    // List available namespaces
    let namespaces = manager.list_namespaces().await?;
    println!("Available namespaces:");
    for ns in &namespaces {
        println!("  - {}", ns);
        
        // List tables in each namespace
        let tables = manager.list_tables(ns).await?;
        for table in tables {
            println!("    - {}", table);
        }
    }
    
    // Register multiple tables for cross-table queries
    if namespaces.contains(&"sales".to_string()) {
        manager.register_table("customers", "sales").await?;
        manager.register_table("orders", "sales").await?;
        manager.register_table("products", "sales").await?;
        
        // Perform a join query across tables
        let results = manager.sql(
            "SELECT 
                c.customer_name,
                p.product_name,
                o.order_date,
                o.quantity,
                o.amount
             FROM orders o
             JOIN customers c ON o.customer_id = c.id
             JOIN products p ON o.product_id = p.id
             WHERE o.order_date >= '2024-01-01'
             ORDER BY o.order_date DESC
             LIMIT 100"
        ).await?;
        
        // Display results
        results.show().await?;
    } else {
        println!("\nNote: 'sales' namespace not found. This example assumes a REST catalog with specific tables.");
        println!("To set up a test REST catalog, you can use the Iceberg REST catalog reference implementation.");
        println!("\nExample setup:");
        println!("  1. Run Iceberg REST catalog server (e.g., using Tabular's implementation)");
        println!("  2. Configure S3 or compatible object storage");
        println!("  3. Create tables using PyIceberg or Spark");
    }
    
    Ok(())
}

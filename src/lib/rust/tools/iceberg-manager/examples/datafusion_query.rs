//! Example: Query Iceberg tables with DataFusion SQL

use anyhow::Result;
use iceberg_manager::{CatalogConfig, TableManager};

#[tokio::main]
async fn main() -> Result<()> {
    // Create catalog configuration
    let catalog_config = CatalogConfig::Memory {
        warehouse: "/tmp/iceberg-warehouse".to_string(),
    };
    
    // Create table manager
    let manager = TableManager::new(catalog_config).await?;
    
    println!("DataFusion Query Example");
    println!("========================\n");
    
    // Check if the catalog has any data
    let namespaces = manager.list_namespaces().await?;
    
    if namespaces.is_empty() {
        println!("No namespaces found in the catalog.");
        println!("\nTo use this example:");
        println!("1. First run the 'create_and_insert' example to create tables");
        println!("2. Insert data using Apache Spark or PyIceberg");
        println!("3. Then run this example to query the data");
        return Ok(());
    }
    
    // Find a table to query
    let mut found_table = false;
    for ns in &namespaces {
        let tables = manager.list_tables(ns).await?;
        if !tables.is_empty() {
            let table = &tables[0];
            println!("Found table: {}.{}", ns, table);
            
            // Register the table with DataFusion
            manager.register_table(table, ns).await?;
            
            // Query the table
            let query = format!("SELECT * FROM {} LIMIT 10", table);
            println!("\nRunning query: {}", query);
            
            let df = manager.sql(&query).await?;
            df.show().await?;
            
            // Show table statistics
            let count_query = format!("SELECT COUNT(*) as row_count FROM {}", table);
            println!("\nRunning count query: {}", count_query);
            
            let count_df = manager.sql(&count_query).await?;
            count_df.show().await?;
            
            found_table = true;
            break;
        }
    }
    
    if !found_table {
        println!("No tables found in any namespace.");
    } else {
        println!("\nDataFusion SQL capabilities demonstrated!");
        println!("You can run complex queries including:");
        println!("- JOINs between multiple tables");
        println!("- Aggregations with GROUP BY");
        println!("- Window functions");
        println!("- CTEs (Common Table Expressions)");
        println!("- And much more!");
    }
    
    Ok(())
}

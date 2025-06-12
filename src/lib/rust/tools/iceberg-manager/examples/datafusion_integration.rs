use anyhow::Result;
use iceberg_manager::{TableManager, CatalogConfig};

/// This example demonstrates how to use DataFusion with Iceberg tables
#[tokio::main]
async fn main() -> Result<()> {
    println!("DataFusion Integration Example");
    println!("==============================\n");
    
    // Create catalog configuration for memory catalog
    let config = CatalogConfig::Memory {
        warehouse: "/tmp/iceberg-warehouse".to_string(),
    };
    
    // Initialize the table manager
    let manager = TableManager::new(config).await?;
    
    println!("DataFusion SQL capabilities:");
    println!("- Complex queries with JOINs, GROUP BY, ORDER BY");
    println!("- Window functions");
    println!("- Aggregate functions");
    println!("- CTEs (Common Table Expressions)");
    println!("- And much more!");
    
    println!("\nExample SQL queries you could run:");
    println!("```sql");
    println!("-- Basic select");
    println!("SELECT * FROM events WHERE timestamp > '2024-01-01' LIMIT 100;");
    println!();
    println!("-- Aggregation");
    println!("SELECT user_id, COUNT(*) as event_count");
    println!("FROM events");
    println!("GROUP BY user_id");
    println!("ORDER BY event_count DESC;");
    println!();
    println!("-- Window functions");
    println!("SELECT user_id, event_type, timestamp,");
    println!("       ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY timestamp) as event_sequence");
    println!("FROM events;");
    println!("```");
    
    // List available namespaces
    println!("\nAvailable namespaces:");
    let namespaces = manager.list_namespaces().await?;
    for ns in &namespaces {
        println!("  - {}", ns);
        
        // List tables in each namespace
        let tables = manager.list_tables(&ns).await?;
        if !tables.is_empty() {
            println!("    Tables:");
            for table in tables {
                println!("      - {}", table);
            }
        }
    }
    
    // If there are tables, demonstrate registering and querying
    if !namespaces.is_empty() {
        let first_ns = &namespaces[0];
        let tables = manager.list_tables(first_ns).await?;
        
        if !tables.is_empty() {
            let table_name = &tables[0];
            println!("\nDemonstrating DataFusion integration with table: {}.{}", first_ns, table_name);
            
            // Register the table with DataFusion
            match manager.register_table(table_name, first_ns).await {
                Ok(_) => {
                    println!("✓ Table registered with DataFusion");
                    
                    // Run a simple query
                    let query = format!("SELECT COUNT(*) as row_count FROM {}", table_name);
                    println!("\nRunning query: {}", query);
                    
                    match manager.sql(&query).await {
                        Ok(df) => {
                            let batches = df.collect().await?;
                            let formatted = datafusion::arrow::util::pretty::pretty_format_batches(&batches)?;
                            println!("{}", formatted);
                        }
                        Err(e) => println!("Query failed: {}", e),
                    }
                }
                Err(e) => println!("Failed to register table: {}", e),
            }
        }
    }
    
    Ok(())
}

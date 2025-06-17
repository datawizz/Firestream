//! Example: Seeding TPC benchmark data into Iceberg tables

use anyhow::Result;
use iceberg_manager::{
    CatalogConfig, TableManager, TpcSeeder, TpcSeederConfig, TpcBenchmark
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Create catalog configuration
    let catalog_config = CatalogConfig::Memory {
        warehouse: "/tmp/iceberg-tpc-demo".to_string(),
    };
    
    // Create table manager
    let table_manager = Arc::new(TableManager::new(catalog_config).await?);
    
    // Create namespaces for our benchmark data
    table_manager.create_namespace("tpch", HashMap::new()).await?;
    table_manager.create_namespace("tpcds", HashMap::new()).await?;
    
    // Configure the seeder
    let config = TpcSeederConfig {
        batch_size: 50_000,  // Process 50k rows at a time
        show_progress: true,  // Show progress indicators
    };
    
    let seeder = TpcSeeder::new(table_manager.clone(), config)?;
    
    // Example 1: Preview what will be migrated
    println!("=== Previewing TPC-H migration ===");
    let tpch_preview = seeder.preview_migration(TpcBenchmark::TpcH, 0.1).await?;
    println!("TPC-H tables at scale factor 0.1:");
    for info in tpch_preview {
        println!("  {} - {} rows, {} columns", info.name, info.row_count, info.column_count);
    }
    println!();
    
    // Example 2: Seed TPC-H data
    println!("=== Seeding TPC-H data ===");
    seeder.seed_tpch("tpch", 0.1).await?;
    println!("TPC-H data seeding completed!\n");
    
    // Example 3: Query the seeded data
    println!("=== Querying TPC-H data ===");
    
    // Register tables with DataFusion
    table_manager.register_table("customer", "tpch").await?;
    table_manager.register_table("orders", "tpch").await?;
    table_manager.register_table("nation", "tpch").await?;
    table_manager.register_table("region", "tpch").await?;
    
    // Run some example queries
    let queries = vec![
        "SELECT COUNT(*) as total_customers FROM customer",
        "SELECT COUNT(*) as total_orders FROM orders",
        "SELECT n_name, n_regionkey FROM nation ORDER BY n_name LIMIT 5",
        "SELECT r_name, r_regionkey FROM region ORDER BY r_regionkey",
    ];
    
    for query in queries {
        println!("\nQuery: {}", query);
        let df = table_manager.sql(query).await?;
        df.show().await?;
    }
    
    // Example 4: More complex TPC-H query
    println!("\n=== Complex TPC-H Query ===");
    let complex_query = r#"
        SELECT 
            n.n_name as nation,
            COUNT(DISTINCT c.c_custkey) as customer_count
        FROM customer c
        JOIN nation n ON c.c_nationkey = n.n_nationkey
        JOIN region r ON n.n_regionkey = r.r_regionkey
        WHERE r.r_name = 'AMERICA'
        GROUP BY n.n_name
        ORDER BY customer_count DESC
    "#;
    
    // First register the region table
    table_manager.register_table("region", "tpch").await?;
    
    println!("Customers by nation in AMERICA region:");
    let df = table_manager.sql(complex_query).await?;
    df.show().await?;
    
    // Example 5: Table statistics
    println!("\n=== Table Statistics ===");
    let stats = table_manager.get_table_stats("tpch", "lineitem").await?;
    println!("Lineitem table stats:");
    println!("{}", stats.display());
    
    // Example 6: Schema inspection
    println!("\n=== Schema Inspection ===");
    let schema = table_manager.get_table_schema("tpch", "customer").await?;
    println!("Customer table schema:");
    println!("{}", schema.display());
    
    Ok(())
}

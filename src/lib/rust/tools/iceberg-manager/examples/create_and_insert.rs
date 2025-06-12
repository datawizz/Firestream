//! Example: Create tables and insert data

use anyhow::Result;
use iceberg_manager::{
    CatalogConfig, TableManager, Schema, NestedField, 
    Type, PrimitiveType, UnboundPartitionSpec, Transform
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Create catalog configuration for memory catalog
    let catalog_config = CatalogConfig::Memory {
        warehouse: "/tmp/iceberg-warehouse".to_string(),
    };
    
    // Create table manager
    let manager = TableManager::new(catalog_config).await?;
    
    // Create a namespace
    manager.create_namespace("analytics", HashMap::new()).await?;
    
    // Define schema for events table using the official API
    let schema = Schema::builder()
        .with_schema_id(1)
        .with_fields(vec![
            NestedField::required(1, "event_id", Type::Primitive(PrimitiveType::Uuid))
                .with_doc("Unique event identifier")
                .into(),
            NestedField::required(2, "user_id", Type::Primitive(PrimitiveType::Long))
                .with_doc("User who triggered the event")
                .into(),
            NestedField::required(3, "event_type", Type::Primitive(PrimitiveType::String))
                .with_doc("Type of event")
                .into(),
            NestedField::optional(4, "properties", Type::Primitive(PrimitiveType::String))
                .with_doc("JSON properties")
                .into(),
            NestedField::required(5, "timestamp", Type::Primitive(PrimitiveType::Timestamptz))
                .with_doc("When the event occurred")
                .into(),
        ])
        .build()?;
    
    // Create partition spec - partition by day
    let partition_spec = UnboundPartitionSpec::builder()
        .add_partition_field(5, "event_day".to_string(), Transform::Day)?
        .build();
    
    // Create the table
    manager.create_table(
        "analytics",
        "events",
        schema,
        Some(partition_spec),
    ).await?;
    
    println!("Created partitioned events table");
    
    // Note: DataFusion with Iceberg tables is read-only at this time
    // Writing data requires using Apache Spark, PyIceberg, or other Iceberg writers
    
    println!("\nTo insert data into this table, you would need to use:");
    println!("- Apache Spark with Iceberg support");
    println!("- PyIceberg Python library");
    println!("- Java/Scala Iceberg APIs");
    
    println!("\nExample Spark SQL to insert data:");
    println!("```sql");
    println!("INSERT INTO analytics.events VALUES");
    println!("  (uuid(), 1001, 'page_view', '{{\"page\": \"/home\"}}', timestamp '2024-01-15 10:30:00'),");
    println!("  (uuid(), 1002, 'click', '{{\"button\": \"signup\"}}', timestamp '2024-01-15 11:45:00'),");
    println!("  (uuid(), 1001, 'purchase', '{{\"item\": \"widget\", \"price\": 29.99}}', timestamp '2024-01-16 09:15:00')");
    println!("```");
    
    println!("\nExample query you could run after inserting data:");
    println!("```sql");
    println!("SELECT ");
    println!("  event_type,");
    println!("  COUNT(*) as event_count,");
    println!("  DATE(timestamp) as event_date");
    println!("FROM events ");
    println!("GROUP BY event_type, DATE(timestamp)");
    println!("ORDER BY event_date, event_type");
    println!("```");
    
    // Show table statistics
    let stats = manager.get_table_stats("analytics", "events").await?;
    println!("\nTable Statistics:");
    println!("{}", stats.display());
    
    Ok(())
}

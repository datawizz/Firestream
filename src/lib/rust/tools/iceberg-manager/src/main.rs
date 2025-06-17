use anyhow::Result;
use clap::{Parser, Subcommand};
use iceberg_manager::{
    CatalogConfig, TableManager, Schema, NestedField, Type, PrimitiveType,
    UnboundPartitionSpec, Transform,
    TpcSeeder, TpcSeederConfig, TpcBenchmark,
};
use iceberg::spec::NestedFieldRef;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "iceberg-manager")]
#[command(about = "A CLI tool for managing Apache Iceberg tables with DataFusion integration", long_about = None)]
struct Cli {
    /// Catalog type (memory or rest)
    #[arg(short = 't', long, default_value = "memory")]
    catalog_type: String,
    
    /// For REST: URI endpoint
    #[arg(short, long)]
    uri: Option<String>,
    
    /// Warehouse location
    #[arg(short = 'w', long, default_value = "/tmp/iceberg-warehouse")]
    warehouse: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Namespace operations
    Namespace {
        #[command(subcommand)]
        action: NamespaceAction,
    },
    /// Table operations
    Table {
        #[command(subcommand)]
        action: TableAction,
    },
    /// SQL query operations
    Sql {
        /// SQL query to execute
        query: String,
    },
    /// Seed TPC benchmark data
    Seed {
        #[command(subcommand)]
        action: SeedAction,
    },
}

#[derive(Subcommand)]
enum SeedAction {
    /// Seed TPC-H data
    TpcH {
        /// Target namespace
        namespace: String,
        /// Scale factor (1, 10, 100, etc.)
        #[arg(short, long, default_value = "1")]
        scale_factor: f64,
        /// Batch size for reading large tables
        #[arg(short, long, default_value = "100000")]
        batch_size: usize,
        /// Preview what will be migrated without actually doing it
        #[arg(long)]
        dry_run: bool,
    },
    /// Seed TPC-DS data
    TpcDs {
        /// Target namespace
        namespace: String,
        /// Scale factor
        #[arg(short, long, default_value = "1")]
        scale_factor: f64,
        /// Batch size for reading large tables
        #[arg(short, long, default_value = "100000")]
        batch_size: usize,
        /// Preview what will be migrated without actually doing it
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum NamespaceAction {
    /// List all namespaces
    List,
    /// Create a new namespace
    Create {
        /// Namespace name (e.g., "prod.analytics")
        name: String,
    },
}

#[derive(Subcommand)]
enum TableAction {
    /// List tables in a namespace
    List {
        /// Namespace name
        namespace: String,
    },
    /// Create a new table
    Create {
        /// Namespace name
        namespace: String,
        /// Table name
        table: String,
        /// Schema type (simple, user, events)
        #[arg(short, long, default_value = "simple")]
        schema_type: String,
        /// Partition by field (optional)
        #[arg(short, long)]
        partition_by: Option<String>,
    },
    /// Show table schema
    Schema {
        /// Namespace name
        namespace: String,
        /// Table name
        table: String,
    },
    /// Preview table data
    Preview {
        /// Namespace name
        namespace: String,
        /// Table name
        table: String,
        /// Number of rows to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Show table statistics
    Stats {
        /// Namespace name
        namespace: String,
        /// Table name
        table: String,
    },
    /// Drop a table
    Drop {
        /// Namespace name
        namespace: String,
        /// Table name
        table: String,
    },
}

fn create_schema(schema_type: &str) -> Result<Schema> {
    let fields = match schema_type {
        "user" => vec![
            NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)),
            NestedField::required(2, "username", Type::Primitive(PrimitiveType::String)),
            NestedField::optional(3, "email", Type::Primitive(PrimitiveType::String)),
            NestedField::optional(4, "full_name", Type::Primitive(PrimitiveType::String)),
            NestedField::required(5, "created_at", Type::Primitive(PrimitiveType::Timestamptz)),
            NestedField::optional(6, "updated_at", Type::Primitive(PrimitiveType::Timestamptz)),
        ],
        "events" => vec![
            NestedField::required(1, "event_id", Type::Primitive(PrimitiveType::Uuid)),
            NestedField::required(2, "user_id", Type::Primitive(PrimitiveType::Long)),
            NestedField::required(3, "event_type", Type::Primitive(PrimitiveType::String)),
            NestedField::optional(4, "event_data", Type::Primitive(PrimitiveType::String)),
            NestedField::required(5, "timestamp", Type::Primitive(PrimitiveType::Timestamptz)),
            NestedField::optional(6, "session_id", Type::Primitive(PrimitiveType::String)),
        ],
        _ => vec![
            NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)),
            NestedField::required(2, "data", Type::Primitive(PrimitiveType::String)),
        ],
    };
    
    Schema::builder()
        .with_schema_id(1)
        .with_fields(fields.into_iter().map(|f| f.into()).collect::<Vec<NestedFieldRef>>())
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build schema: {}", e))
}

fn create_partition_spec(partition_by: &str, schema: &Schema) -> Result<UnboundPartitionSpec> {
    // Find the field in the schema
    let struct_type = schema.as_struct();
    let field_id = struct_type.fields()
        .iter()
        .find(|f| f.name == partition_by)
        .map(|f| f.id)
        .ok_or_else(|| anyhow::anyhow!("Field '{}' not found in schema", partition_by))?;
    
    // Determine the transform based on the field name
    let transform = if partition_by.contains("date") || partition_by.contains("time") {
        Transform::Day
    } else {
        Transform::Identity
    };
    
    Ok(UnboundPartitionSpec::builder()
        .add_partition_field(field_id, partition_by.to_string(), transform)?
        .build())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    // Create catalog configuration
    let catalog_config = match cli.catalog_type.as_str() {
        "rest" => {
            let uri = cli.uri
                .ok_or_else(|| anyhow::anyhow!("URI required for REST catalog"))?;
            CatalogConfig::Rest {
                uri,
                warehouse: cli.warehouse,
                props: HashMap::new(),
            }
        }
        _ => CatalogConfig::Memory {
            warehouse: cli.warehouse,
        }
    };
    
    // Create table manager
    let manager = TableManager::new(catalog_config).await?;
    
    match cli.command {
        Commands::Namespace { action } => match action {
            NamespaceAction::List => {
                let namespaces = manager.list_namespaces().await?;
                if namespaces.is_empty() {
                    println!("No namespaces found");
                } else {
                    println!("Namespaces:");
                    for ns in namespaces {
                        println!("  {}", ns);
                    }
                }
            }
            NamespaceAction::Create { name } => {
                manager.create_namespace(&name, HashMap::new()).await?;
                println!("Created namespace: {}", name);
            }
        },
        Commands::Table { action } => match action {
            TableAction::List { namespace } => {
                let tables = manager.list_tables(&namespace).await?;
                if tables.is_empty() {
                    println!("No tables found in namespace: {}", namespace);
                } else {
                    println!("Tables in {}:", namespace);
                    for table in tables {
                        println!("  {}", table);
                    }
                }
            }
            TableAction::Create { namespace, table, schema_type, partition_by } => {
                let schema = create_schema(&schema_type)?;
                
                let partition_spec = if let Some(ref field) = partition_by {
                    Some(create_partition_spec(&field, &schema)?)
                } else {
                    None
                };
                
                manager.create_table(
                    &namespace,
                    &table,
                    schema,
                    partition_spec,
                ).await?;
                
                println!("Created table {}.{} with {} schema", namespace, table, schema_type);
                if let Some(ref field) = partition_by {
                    println!("  Partitioned by: {}", field);
                }
            }
            TableAction::Schema { namespace, table } => {
                let schema = manager.get_table_schema(&namespace, &table).await?;
                println!("Schema for {}.{}:", namespace, table);
                println!("{}", schema.display());
            }
            TableAction::Preview { namespace, table, limit } => {
                let preview = manager.preview_table(&namespace, &table, Some(limit)).await?;
                println!("Preview of {}.{} (showing {} rows):", namespace, table, preview.row_count);
                println!("{}", preview.display()?);
            }
            TableAction::Stats { namespace, table } => {
                let stats = manager.get_table_stats(&namespace, &table).await?;
                println!("Statistics for {}.{}:", namespace, table);
                println!("{}", stats.display());
            }
            TableAction::Drop { namespace, table } => {
                manager.drop_table(&namespace, &table).await?;
                println!("Dropped table {}.{}", namespace, table);
            }
        },
        Commands::Sql { query } => {
            println!("Executing SQL: {}", query);
            let df = manager.sql(&query).await?;
            
            // Collect and display results
            let batches = df.collect().await?;
            if batches.is_empty() {
                println!("Query returned no results");
            } else {
                let formatted = datafusion::arrow::util::pretty::pretty_format_batches(&batches)?;
                println!("{}", formatted);
            }
        }
        Commands::Seed { action } => {
            let config = match &action {
                SeedAction::TpcH { batch_size, .. } | SeedAction::TpcDs { batch_size, .. } => {
                    TpcSeederConfig {
                        batch_size: *batch_size,
                        show_progress: true,
                    }
                }
            };
            
            let seeder = TpcSeeder::new(std::sync::Arc::new(manager), config)?;
            
            match action {
                SeedAction::TpcH { namespace, scale_factor, dry_run, .. } => {
                    if dry_run {
                        println!("Preview of TPC-H migration (scale factor: {})", scale_factor);
                        let table_infos = seeder.preview_migration(TpcBenchmark::TpcH, scale_factor).await?;
                        println!("Tables to be migrated:");
                        for info in table_infos {
                            println!("  - {}: {} rows, {} columns", info.name, info.row_count, info.column_count);
                        }
                    } else {
                        println!("Seeding TPC-H data at scale factor {} into namespace '{}'", scale_factor, namespace);
                        seeder.seed_tpch(&namespace, scale_factor).await?;
                        println!("\nTPC-H data seeding completed successfully!");
                    }
                }
                SeedAction::TpcDs { namespace, scale_factor, dry_run, .. } => {
                    if dry_run {
                        println!("Preview of TPC-DS migration (scale factor: {})", scale_factor);
                        let table_infos = seeder.preview_migration(TpcBenchmark::TpcDs, scale_factor).await?;
                        println!("Tables to be migrated:");
                        for info in table_infos {
                            println!("  - {}: {} rows, {} columns", info.name, info.row_count, info.column_count);
                        }
                    } else {
                        println!("Seeding TPC-DS data at scale factor {} into namespace '{}'", scale_factor, namespace);
                        seeder.seed_tpcds(&namespace, scale_factor).await?;
                        println!("\nTPC-DS data seeding completed successfully!");
                    }
                }
            }
        }
    }
    
    Ok(())
}

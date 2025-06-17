# Iceberg Manager

A Rust library and CLI tool for managing Apache Iceberg tables with DataFusion integration.

## Features

- **DataFusion Integration**: Query Iceberg tables using SQL through DataFusion v47
- **Multiple Catalog Support**: Memory and REST catalog implementations
- **Table Management**: Create, list, and drop tables
- **Schema Management**: Create and inspect table schemas
- **SQL Queries**: Execute SQL queries against Iceberg tables (read-only)
- **TPC Benchmark Data**: Generate and load TPC-H and TPC-DS benchmark data
- **CLI Tool**: Command-line interface for table management

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
iceberg-manager = { path = "../path/to/iceberg-manager" }
```

## Library Usage

### Basic Example

```rust
use iceberg_manager::{TableManager, CatalogConfig, Schema, NestedField, Type, PrimitiveType};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a memory catalog
    let config = CatalogConfig::Memory {
        warehouse: "/tmp/iceberg-warehouse".to_string(),
    };
    
    // Initialize the table manager
    let manager = TableManager::new(config).await?;
    
    // Create a namespace
    manager.create_namespace("prod", HashMap::new()).await?;
    
    // Create a table with schema
    let schema = Schema::builder()
        .with_schema_id(1)
        .with_fields(vec![
            NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)).into(),
            NestedField::required(2, "username", Type::Primitive(PrimitiveType::String)).into(),
            NestedField::optional(3, "email", Type::Primitive(PrimitiveType::String)).into(),
        ])
        .build()?;
    
    manager.create_table("prod", "users", schema, None).await?;
    
    // Query with SQL
    manager.register_table("users", "prod").await?;
    let df = manager.sql("SELECT * FROM users LIMIT 10").await?;
    df.show().await?;
    
    Ok(())
}
```

### Using the Test Schema Helper

```rust
use iceberg_manager::{TableManager, CatalogConfig, create_test_schema};

// Use the built-in test schema helper
let schema = create_test_schema()?;
// Creates a schema with: id (long), name (string), email (string?), created_at (timestamp)

manager.create_table("test", "users", schema, None).await?;
```

### REST Catalog Example

```rust
use iceberg_manager::{TableManager, CatalogConfig};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let mut props = HashMap::new();
    // Add authentication if needed
    // props.insert("header.Authorization".to_string(), "Bearer <token>".to_string());
    
    let config = CatalogConfig::Rest {
        uri: "http://localhost:8181".to_string(),
        warehouse: "s3://my-bucket/warehouse".to_string(),
        props,
    };
    
    let manager = TableManager::new(config).await?;
    // ... use the manager
    Ok(())
}
```

## CLI Usage

### Building the CLI

```bash
cargo build --release --bin iceberg-manager
```

### Basic Commands

```bash
# List namespaces
iceberg-manager namespace list

# Create a namespace
iceberg-manager namespace create prod.analytics

# List tables in a namespace
iceberg-manager table list prod

# Create a table
iceberg-manager table create prod users --schema-type user

# Create a partitioned table
iceberg-manager table create prod events --schema-type events --partition-by timestamp

# Show table schema
iceberg-manager table schema prod users

# Preview table data
iceberg-manager table preview prod users --limit 20

# Show table statistics
iceberg-manager table stats prod users

# Execute SQL query
iceberg-manager sql "SELECT COUNT(*) FROM users"

# Drop a table
iceberg-manager table drop prod users
```

### TPC Benchmark Data Seeding

```bash
# Preview what TPC-H tables will be created (dry run)
iceberg-manager seed tpc-h test --scale-factor 1 --dry-run

# Seed TPC-H data at scale factor 1
iceberg-manager seed tpc-h tpch --scale-factor 1

# Seed TPC-H with custom batch size for large scale factors
iceberg-manager seed tpc-h tpch --scale-factor 100 --batch-size 50000

# Seed TPC-DS data
iceberg-manager seed tpc-ds tpcds --scale-factor 10
```

### Using REST Catalog

```bash
# Connect to REST catalog
iceberg-manager --catalog-type rest --uri http://localhost:8181 --warehouse s3://bucket/warehouse namespace list
```

## Supported Schema Types

The CLI supports several predefined schema types:

- **simple**: Basic id and data fields
- **user**: User table with id, username, email, full_name, created_at, updated_at  
- **events**: Event tracking table with event_id, user_id, event_type, event_data, timestamp, session_id

## DataFusion SQL Support

Once tables are registered, you can use the full power of DataFusion SQL for reading data:

```sql
-- Basic queries
SELECT * FROM users WHERE created_at > '2024-01-01';

-- Aggregations
SELECT COUNT(*), DATE(created_at) as signup_date 
FROM users 
GROUP BY DATE(created_at);

-- Joins (after registering multiple tables)
SELECT u.username, COUNT(e.event_id) as event_count
FROM users u
LEFT JOIN events e ON u.id = e.user_id
GROUP BY u.username;

-- Window functions
SELECT 
    username,
    created_at,
    ROW_NUMBER() OVER (ORDER BY created_at) as user_number
FROM users;

-- CTEs (Common Table Expressions)
WITH active_users AS (
    SELECT DISTINCT user_id 
    FROM events 
    WHERE timestamp > '2024-01-01'
)
SELECT u.* 
FROM users u
JOIN active_users a ON u.id = a.user_id;
```

**Note**: DataFusion integration currently supports read-only queries. To write data to Iceberg tables, use Apache Spark, PyIceberg, or other Iceberg-compatible writers.

## Architecture

The library is built on top of the official Apache Iceberg Rust implementation (v0.5.1) and provides:

1. **TableManager**: High-level API for table operations
2. **CatalogProvider**: DataFusion v47 catalog integration via `iceberg-datafusion`
3. **Error Handling**: Comprehensive error types with detailed context
4. **Type Re-exports**: Common Iceberg types for convenience

### Key Dependencies

- `iceberg` v0.5.1 - Official Apache Iceberg Rust implementation
- `iceberg-datafusion` v0.5.1 - DataFusion integration
- `datafusion` v47 - SQL query engine
- `arrow` v55 - Columnar data format
- `duckdb` v1.3 - For TPC-H and TPC-DS data generation

## Creating Tables with Partitions

```rust
use iceberg_manager::{UnboundPartitionSpec, Transform};

// Create a partitioned table
let partition_spec = UnboundPartitionSpec::builder()
    .add_partition_field(5, "event_day".to_string(), Transform::Day)?
    .build();

manager.create_table("analytics", "events", schema, Some(partition_spec)).await?;
```

## Migration from Unofficial Iceberg-Rust

This library now uses the official Apache Iceberg Rust implementation. Key changes:

- Uses `iceberg` crate instead of `iceberg-rust`
- `NamespaceIdent` instead of `Namespace`
- `TableIdent` instead of `Identifier`
- `FileIO` instead of `ObjectStore`
- Simplified catalog integration with `iceberg-datafusion`
- Compatible with DataFusion v47 and Arrow v55

## Limitations

- **Read-only SQL**: DataFusion integration currently supports only reading data
- **No File Catalog**: Use Memory catalog for local development
- **Limited Catalog Support**: Currently supports Memory and REST catalogs

## TPC Benchmark Data Generation

The library includes built-in support for generating TPC-H and TPC-DS benchmark data directly into Iceberg tables:

```rust
use iceberg_manager::{TpcSeeder, TpcSeederConfig, TpcBenchmark};
use std::sync::Arc;

// Configure the seeder
let config = TpcSeederConfig {
    batch_size: 100_000,  // Process 100k rows at a time
    show_progress: true,  // Show progress indicators
};

let seeder = TpcSeeder::new(Arc::new(manager), config)?;

// Preview what will be created
let preview = seeder.preview_migration(TpcBenchmark::TpcH, 1.0).await?;

// Seed TPC-H data at scale factor 1
seeder.seed_tpch("tpch", 1.0).await?;

// Seed TPC-DS data at scale factor 10
seeder.seed_tpcds("tpcds", 10.0).await?;
```

### Features:
- **Dynamic Table Discovery**: Automatically discovers and migrates all tables generated by DuckDB
- **Batch Processing**: Handles large datasets efficiently with configurable batch sizes
- **Progress Tracking**: Optional progress indicators for long-running migrations
- **Schema Conversion**: Automatic conversion from DuckDB/Arrow schemas to Iceberg schemas
- **Preview Mode**: Dry-run capability to see what will be migrated before execution

## Examples

See the `examples/` directory for more detailed examples:

- `basic_usage.rs` - Basic table operations
- `datafusion_integration.rs` - SQL query examples
- `create_and_insert.rs` - Table creation with schemas
- `rest_catalog.rs` - REST catalog usage
- `seed_tpc_data.rs` - TPC benchmark data seeding

## Testing

Run the test suite:

```bash
cargo test
```

The test suite includes:
- Unit tests for core functionality
- Integration tests for catalog operations
- Example compilation tests

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the same terms as the parent Firestream project.

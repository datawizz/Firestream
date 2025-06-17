//! TPC benchmark data seeder for Iceberg tables

use crate::error::{Error, Result};
use crate::schema_converter::SchemaConverter;
use crate::table_manager::TableManager;
use arrow::record_batch::RecordBatch;
use duckdb::{Connection, Result as DuckDbResult};
use iceberg::writer::base_writer::data_file_writer::DataFileWriterBuilder;
use iceberg::writer::file_writer::location_generator::{
    DefaultFileNameGenerator, DefaultLocationGenerator,
};
use iceberg::writer::file_writer::ParquetWriterBuilder;
use iceberg::writer::{IcebergWriter, IcebergWriterBuilder};
use iceberg::TableIdent;
use std::sync::Arc;

/// TPC benchmark type
#[derive(Debug, Clone, Copy)]
pub enum TpcBenchmark {
    TpcH,
    TpcDs,
}

/// Configuration for TPC data seeding
#[derive(Debug, Clone)]
pub struct TpcSeederConfig {
    /// Batch size for reading large tables
    pub batch_size: usize,
    /// Whether to show progress indicators
    pub show_progress: bool,
}

impl Default for TpcSeederConfig {
    fn default() -> Self {
        Self {
            batch_size: 100_000,
            show_progress: true,
        }
    }
}

/// TPC benchmark data seeder
pub struct TpcSeeder {
    duckdb_conn: Connection,
    table_manager: Arc<TableManager>,
    config: TpcSeederConfig,
}

impl TpcSeeder {
    /// Create a new TPC seeder
    pub fn new(table_manager: Arc<TableManager>, config: TpcSeederConfig) -> Result<Self> {
        let duckdb_conn = Connection::open_in_memory()
            .map_err(|e| Error::InvalidOperation(format!("Failed to create DuckDB connection: {}", e)))?;
        
        Ok(Self {
            duckdb_conn,
            table_manager,
            config,
        })
    }
    
    /// Seed TPC-H benchmark data
    pub async fn seed_tpch(&self, namespace: &str, scale_factor: f64) -> Result<()> {
        // Install and load TPC-H extension
        self.duckdb_conn
            .execute("INSTALL tpch", [])
            .map_err(|e| Error::InvalidOperation(format!("Failed to install TPC-H: {}", e)))?;
        self.duckdb_conn
            .execute("LOAD tpch", [])
            .map_err(|e| Error::InvalidOperation(format!("Failed to load TPC-H: {}", e)))?;
        
        // Generate data
        self.duckdb_conn
            .execute(&format!("CALL dbgen(sf = {})", scale_factor), [])
            .map_err(|e| Error::InvalidOperation(format!("Failed to generate TPC-H data: {}", e)))?;
        
        // Discover and migrate tables
        let tables = self.discover_tables()?;
        
        if self.config.show_progress {
            println!("Found {} TPC-H tables to migrate", tables.len());
        }
        
        for table_name in tables {
            if self.config.show_progress {
                println!("\nProcessing table: {}", table_name);
            }
            self.migrate_table(namespace, &table_name).await?;
        }
        
        Ok(())
    }
    
    /// Seed TPC-DS benchmark data
    pub async fn seed_tpcds(&self, namespace: &str, scale_factor: f64) -> Result<()> {
        // Install and load TPC-DS extension
        self.duckdb_conn
            .execute("INSTALL tpcds", [])
            .map_err(|e| Error::InvalidOperation(format!("Failed to install TPC-DS: {}", e)))?;
        self.duckdb_conn
            .execute("LOAD tpcds", [])
            .map_err(|e| Error::InvalidOperation(format!("Failed to load TPC-DS: {}", e)))?;
        
        // Generate data
        self.duckdb_conn
            .execute(&format!("CALL dsdgen(sf = {})", scale_factor), [])
            .map_err(|e| Error::InvalidOperation(format!("Failed to generate TPC-DS data: {}", e)))?;
        
        // Discover and migrate tables
        let tables = self.discover_tables()?;
        
        if self.config.show_progress {
            println!("Found {} TPC-DS tables to migrate", tables.len());
        }
        
        for table_name in tables {
            if self.config.show_progress {
                println!("\nProcessing table: {}", table_name);
            }
            self.migrate_table(namespace, &table_name).await?;
        }
        
        Ok(())
    }
    
    /// Preview what tables will be migrated without actually doing it
    pub async fn preview_migration(&self, benchmark: TpcBenchmark, scale_factor: f64) -> Result<Vec<TableInfo>> {
        match benchmark {
            TpcBenchmark::TpcH => {
                self.duckdb_conn.execute("INSTALL tpch", [])?;
                self.duckdb_conn.execute("LOAD tpch", [])?;
                self.duckdb_conn.execute(&format!("CALL dbgen(sf = {})", scale_factor), [])?;
            }
            TpcBenchmark::TpcDs => {
                self.duckdb_conn.execute("INSTALL tpcds", [])?;
                self.duckdb_conn.execute("LOAD tpcds", [])?;
                self.duckdb_conn.execute(&format!("CALL dsdgen(sf = {})", scale_factor), [])?;
            }
        }
        
        let tables = self.discover_tables()?;
        let mut table_infos = Vec::new();
        
        for table_name in tables {
            let row_count = self.get_row_count(&table_name)?;
            let schema = self.get_table_schema(&table_name)?;
            table_infos.push(TableInfo {
                name: table_name,
                row_count,
                column_count: schema.fields().len(),
            });
        }
        
        Ok(table_infos)
    }
    
    /// Discover all tables in DuckDB
    fn discover_tables(&self) -> Result<Vec<String>> {
        let mut stmt = self.duckdb_conn
            .prepare(
                "SELECT table_name 
                 FROM information_schema.tables 
                 WHERE table_schema = 'main' 
                   AND table_type = 'BASE TABLE'
                 ORDER BY table_name"
            )
            .map_err(|e| Error::InvalidOperation(format!("Failed to query tables: {}", e)))?;
        
        let tables = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| Error::InvalidOperation(format!("Failed to map table names: {}", e)))?
            .collect::<DuckDbResult<Vec<String>>>()
            .map_err(|e| Error::InvalidOperation(format!("Failed to collect table names: {}", e)))?;
        
        Ok(tables)
    }
    
    /// Migrate a single table from DuckDB to Iceberg
    async fn migrate_table(&self, namespace: &str, table_name: &str) -> Result<()> {
        // Get table metadata
        let row_count = self.get_row_count(table_name)?;
        if self.config.show_progress {
            println!("  - Row count: {}", row_count);
        }
        
        if row_count == 0 {
            if self.config.show_progress {
                println!("  - Skipping empty table");
            }
            return Ok(());
        }
        
        // Get table schema
        let arrow_schema = self.get_table_schema(table_name)?;
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema)?;
        
        // Create Iceberg table
        if self.config.show_progress {
            println!("  - Creating Iceberg table");
        }
        self.table_manager
            .create_table(namespace, table_name, iceberg_schema, None)
            .await?;
        
        // Extract and write data
        let record_batches = self.extract_table_data(table_name, row_count)?;
        if self.config.show_progress {
            println!("  - Writing {} batches to Iceberg", record_batches.len());
        }
        self.write_to_iceberg(namespace, table_name, record_batches).await?;
        
        if self.config.show_progress {
            println!("  - Completed migration of {}", table_name);
        }
        
        Ok(())
    }
    
    /// Extract table data as Arrow RecordBatches
    fn extract_table_data(&self, table_name: &str, total_rows: usize) -> Result<Vec<RecordBatch>> {
        // Due to DuckDB decimal panic bug, always use the workaround for safety
        // Check if the table has any columns that might cause issues
        match self.check_has_problematic_decimals(table_name) {
            Ok(has_decimals) if has_decimals => {
                return self.extract_table_data_with_decimal_workaround(table_name, total_rows);
            }
            Err(_) => {
                // If we can't check, use workaround to be safe
                return self.extract_table_data_with_decimal_workaround(table_name, total_rows);
            }
            _ => {}
        }
        
        // For tables without decimals, try direct extraction but be ready to fallback
        self.extract_table_data_safe(table_name, total_rows)
    }
    
    /// Safe extraction that won't panic
    fn extract_table_data_safe(&self, table_name: &str, total_rows: usize) -> Result<Vec<RecordBatch>> {
        // Get schema first to check for issues
        let schema = self.get_table_schema(table_name)?;
        
        // Check if any field is a decimal
        for field in schema.fields() {
            if matches!(field.data_type(), arrow::datatypes::DataType::Decimal128(_, _)) {
                return self.extract_table_data_with_decimal_workaround(table_name, total_rows);
            }
        }
        
        // If no decimals in schema, should be safe to query
        let mut all_batches = Vec::new();
        let mut offset = 0;
        
        while offset < total_rows {
            let query = format!(
                "SELECT * FROM {} LIMIT {} OFFSET {}", 
                table_name, self.config.batch_size, offset
            );
            
            let mut stmt = self.duckdb_conn
                .prepare(&query)
                .map_err(|e| Error::InvalidOperation(format!("Failed to prepare query: {}", e)))?;
            
            let batches: Vec<RecordBatch> = stmt
                .query_arrow([])
                .map_err(|e| Error::InvalidOperation(format!("Failed to query arrow: {}", e)))?
                .collect();
            
            all_batches.extend(batches);
            offset += self.config.batch_size;
            
            // Progress indicator
            if self.config.show_progress && total_rows > self.config.batch_size {
                let progress = (offset as f64 / total_rows as f64 * 100.0).min(100.0);
                println!("    Reading data: {:.1}%", progress);
            }
        }
        
        Ok(all_batches)
    }
    
    /// Check if table has decimal columns that might cause Arrow conversion issues
    fn check_has_problematic_decimals(&self, table_name: &str) -> Result<bool> {
        // Try a more direct approach - check PRAGMA table_info
        let mut stmt = self.duckdb_conn
            .prepare(&format!("PRAGMA table_info({})", table_name))
            .map_err(|e| Error::InvalidOperation(format!("Failed to get table info: {}", e)))?;
        
        let rows = stmt
            .query_map([], |row| {
                // Column 2 is the data type
                let data_type: String = row.get(2)?;
                Ok(data_type)
            })
            .map_err(|e| Error::InvalidOperation(format!("Failed to query table info: {}", e)))?;
        
        for row in rows {
            let data_type = row.map_err(|e| Error::InvalidOperation(format!("Failed to read type: {}", e)))?;
            let upper_type = data_type.to_uppercase();
            // Check for any decimal-like types, including parameterized forms
            if upper_type.contains("DECIMAL") 
                || upper_type.contains("NUMERIC") 
                || upper_type.contains("DEC") 
                || upper_type.contains("FIXED") {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Extract table data with decimal workaround by casting decimals to DOUBLE
    fn extract_table_data_with_decimal_workaround(&self, table_name: &str, total_rows: usize) -> Result<Vec<RecordBatch>> {
        // Get schema to build proper select query
        let schema = self.get_table_schema(table_name)?;
        
        // Build SELECT query that casts decimal columns to DOUBLE
        let mut select_items = Vec::new();
        for field in schema.fields() {
            match field.data_type() {
                arrow::datatypes::DataType::Decimal128(_, _) => {
                    select_items.push(format!("CAST({} AS DOUBLE) AS {}", field.name(), field.name()));
                }
                _ => select_items.push(field.name().to_string())
            }
        }
        
        let mut all_batches = Vec::new();
        let mut offset = 0;
        
        while offset < total_rows {
            let query = format!(
                "SELECT {} FROM {} LIMIT {} OFFSET {}", 
                select_items.join(", "),
                table_name, 
                self.config.batch_size, 
                offset
            );
            
            let mut stmt = self.duckdb_conn
                .prepare(&query)
                .map_err(|e| Error::InvalidOperation(format!("Failed to prepare query: {}", e)))?;
            
            let batches: Vec<RecordBatch> = stmt
                .query_arrow([])
                .map_err(|e| Error::InvalidOperation(format!("Failed to query arrow with workaround: {}", e)))?
                .collect();
            
            all_batches.extend(batches);
            offset += self.config.batch_size;
            
            // Progress indicator
            if self.config.show_progress && total_rows > self.config.batch_size {
                let progress = (offset as f64 / total_rows as f64 * 100.0).min(100.0);
                println!("    Reading data: {:.1}% (using decimal workaround)", progress);
            }
        }
        
        Ok(all_batches)
    }
    
    /// Get row count for a table
    fn get_row_count(&self, table_name: &str) -> Result<usize> {
        let mut stmt = self.duckdb_conn
            .prepare(&format!("SELECT COUNT(*) FROM {}", table_name))
            .map_err(|e| Error::InvalidOperation(format!("Failed to prepare count query: {}", e)))?;
        
        stmt.query_row([], |row| row.get(0))
            .map_err(|e| Error::InvalidOperation(format!("Failed to get row count: {}", e)))
    }
    
    /// Get table schema
    fn get_table_schema(&self, table_name: &str) -> Result<arrow::datatypes::Schema> {
        // For TPC-H tables, always use the fallback since they often have decimals
        let tpc_tables = ["lineitem", "orders", "partsupp", "part", "supplier", "customer", "nation", "region"];
        if tpc_tables.iter().any(|&t| table_name.to_lowercase() == t) {
            return self.get_schema_from_information_schema(table_name);
        }
        
        // First try to get the schema directly
        match self.try_get_schema_direct(table_name) {
            Ok(schema) => Ok(schema),
            Err(_) => {
                // For any error, try the fallback
                self.get_schema_from_information_schema(table_name)
            }
        }
    }
    
    /// Try to get schema directly using Arrow interface
    fn try_get_schema_direct(&self, table_name: &str) -> Result<arrow::datatypes::Schema> {
        let mut stmt = self.duckdb_conn
            .prepare(&format!("SELECT * FROM {} LIMIT 0", table_name))
            .map_err(|e| Error::InvalidOperation(format!("Failed to prepare schema query: {}", e)))?;
        
        let batches: Vec<RecordBatch> = stmt
            .query_arrow([])
            .map_err(|e| Error::InvalidOperation(format!("Failed to query schema: {}", e)))?
            .collect();
        
        if batches.is_empty() {
            return Err(Error::InvalidOperation("No schema found".to_string()));
        }
        
        Ok(batches[0].schema().as_ref().clone())
    }
    
    /// Build schema from information_schema to avoid decimal conversion issues
    fn get_schema_from_information_schema(&self, table_name: &str) -> Result<arrow::datatypes::Schema> {
        use arrow::datatypes::{Field as ArrowField, DataType};
        
        // PRAGMA table_info in DuckDB doesn't use quotes
        let query = format!("PRAGMA table_info({})", table_name);
        let mut stmt = self.duckdb_conn
            .prepare(&query)
            .map_err(|e| Error::InvalidOperation(format!("Failed to query table info for {}: {}", table_name, e)))?;
        
        let mut fields = Vec::new();
        
        // First check if we can execute the query
        let rows = stmt
            .query_map([], |row| {
                // PRAGMA table_info returns:
                // 0: cid, 1: name, 2: type, 3: notnull, 4: dflt_value, 5: pk
                Ok((
                    row.get::<_, String>(1)?,     // column_name
                    row.get::<_, String>(2)?,     // data_type
                    row.get::<_, bool>(3)?,        // notnull (false = nullable, true = not null)
                ))
            })
            .map_err(|e| Error::InvalidOperation(format!("Failed to map column info: {}", e)))?;
        
        for row in rows {
            let (col_name, data_type, not_null) = row
                .map_err(|e| Error::InvalidOperation(format!("Failed to read column info: {}", e)))?;
            
            // Parse the data type, handling cases like DECIMAL(10,2)
            let arrow_type = if data_type.to_uppercase().starts_with("DECIMAL") {
                // Parse precision and scale from DECIMAL(p,s)
                let (precision, scale) = if let Some(start) = data_type.find('(') {
                    if let Some(end) = data_type.find(')') {
                        let params = &data_type[start+1..end];
                        let parts: Vec<&str> = params.split(',').collect();
                        let p = parts.get(0).and_then(|s| s.trim().parse::<u8>().ok()).unwrap_or(38);
                        let s = parts.get(1).and_then(|s| s.trim().parse::<i8>().ok()).unwrap_or(0);
                        (p, s)
                    } else {
                        (38, 0)
                    }
                } else {
                    (38, 0)
                };
                DataType::Decimal128(precision, scale)
            } else {
                let upper_type = data_type.to_uppercase();
                // Remove any parentheses for base type matching
                let base_type = upper_type.split('(').next().unwrap_or(&upper_type).trim();
                
                match base_type {
                    "INTEGER" | "INT" | "INT4" => DataType::Int32,
                    "BIGINT" | "INT8" => DataType::Int64,
                    "SMALLINT" | "INT2" => DataType::Int16,
                    "TINYINT" | "INT1" => DataType::Int8,
                    "REAL" | "FLOAT" | "FLOAT4" => DataType::Float32,
                    "DOUBLE" | "DOUBLE PRECISION" | "FLOAT8" => DataType::Float64,
                    "VARCHAR" | "TEXT" | "STRING" => DataType::Utf8,
                    "CHAR" | "BPCHAR" => DataType::Utf8,
                    "BOOLEAN" | "BOOL" => DataType::Boolean,
                    "DATE" => DataType::Date32,
                    "TIME" => DataType::Time64(arrow::datatypes::TimeUnit::Microsecond),
                    "TIMESTAMP" | "TIMESTAMPTZ" => DataType::Timestamp(arrow::datatypes::TimeUnit::Microsecond, None),
                    _ => DataType::Utf8 // Default to string for unknown types
                }
            };
            
            let nullable = !not_null; // false = nullable, true = not null
            fields.push(ArrowField::new(col_name, arrow_type, nullable));
        }
        
        if fields.is_empty() {
            return Err(Error::InvalidOperation(format!("No columns found for table {}", table_name)));
        }
        
        Ok(arrow::datatypes::Schema::new(fields))
    }
    
    /// Write RecordBatches to Iceberg table
    async fn write_to_iceberg(
        &self,
        namespace: &str,
        table_name: &str,
        batches: Vec<RecordBatch>,
    ) -> Result<()> {
        // Parse namespace and create table identifier
        let ns = self.table_manager.parse_namespace(namespace)?;
        let table_ident = TableIdent::new(ns, table_name.to_string());
        
        // Load the table
        let table = self.table_manager
            .catalog
            .load_table(&table_ident)
            .await?;
        
        // Set up writers
        let location_generator = DefaultLocationGenerator::new(table.metadata().clone())
            .map_err(|e| Error::InvalidOperation(format!("Failed to create location generator: {}", e)))?;
        
        let file_name_generator = DefaultFileNameGenerator::new(
            format!("{}_data", table_name),
            None,
            iceberg::spec::DataFileFormat::Parquet,
        );
        
        let parquet_writer_builder = ParquetWriterBuilder::new(
            Default::default(),
            table.metadata().current_schema().clone(),
            table.file_io().clone(),
            location_generator,
            file_name_generator,
        );
        
        let data_file_writer_builder = DataFileWriterBuilder::new(
            parquet_writer_builder,
            None,  // No partition value for unpartitioned tables
            0,     // Default partition spec ID
        );
        let mut writer = data_file_writer_builder
            .build()
            .await
            .map_err(|e| Error::InvalidOperation(format!("Failed to build writer: {}", e)))?;
        
        // Write all batches
        for (idx, batch) in batches.iter().enumerate() {
            writer
                .write(batch.clone())
                .await
                .map_err(|e| Error::InvalidOperation(format!("Failed to write batch: {}", e)))?;
            
            if self.config.show_progress && batches.len() > 10 && idx % 10 == 0 {
                let progress = (idx + 1) as f64 / batches.len() as f64 * 100.0;
                println!("    Writing to Iceberg: {:.1}%", progress);
            }
        }
        
        // Close writer and get data files
        let data_files = writer
            .close()
            .await
            .map_err(|e| Error::InvalidOperation(format!("Failed to close writer: {}", e)))?;
        
        // Create a transaction with the table
        let tx = iceberg::transaction::Transaction::new(&table);
        
        // Use fast append to add the data files
        let mut append = tx.fast_append(None, vec![])
            .map_err(|e| Error::InvalidOperation(format!("Failed to create fast append: {}", e)))?;
        
        // Use add_data_files (plural) which takes an iterator
        append.add_data_files(data_files)
            .map_err(|e| Error::InvalidOperation(format!("Failed to add data files: {}", e)))?;
        
        // Apply the append action
        let tx = append.apply().await
            .map_err(|e| Error::InvalidOperation(format!("Failed to apply append: {}", e)))?;
        
        // Commit the transaction
        tx.commit(self.table_manager.catalog.as_ref())
            .await
            .map_err(|e| Error::InvalidOperation(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(())
    }
}

/// Information about a table to be migrated
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub row_count: usize,
    pub column_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CatalogConfig;
    use std::collections::HashMap;
    use tempfile::TempDir;
    
    async fn create_test_seeder() -> Result<(TpcSeeder, TempDir)> {
        let temp_dir = TempDir::new().unwrap();
        let warehouse_path = temp_dir.path().to_str().unwrap().to_string();
        
        let catalog_config = CatalogConfig::Memory { warehouse: warehouse_path };
        let table_manager = Arc::new(TableManager::new(catalog_config).await?);
        
        // Create test namespace
        table_manager.create_namespace("test", HashMap::new()).await?;
        
        let config = TpcSeederConfig {
            batch_size: 1000,
            show_progress: false,
        };
        
        let seeder = TpcSeeder::new(table_manager, config)?;
        Ok((seeder, temp_dir))
    }
    
    #[tokio::test]
    async fn test_seeder_creation() {
        let (seeder, _temp_dir) = create_test_seeder().await.unwrap();
        assert_eq!(seeder.config.batch_size, 1000);
        assert!(!seeder.config.show_progress);
    }
    
    #[tokio::test]
    async fn test_discover_tables_empty() {
        let (seeder, _temp_dir) = create_test_seeder().await.unwrap();
        let tables = seeder.discover_tables().unwrap();
        assert_eq!(tables.len(), 0);
    }
    
    #[tokio::test]
    #[ignore = "DuckDB has a panic bug with decimal types in Arrow conversion"]
    async fn test_tpch_preview() {
        let (seeder, _temp_dir) = create_test_seeder().await.unwrap();
        
        // Preview TPC-H at scale factor 0.01 (smallest)
        let table_infos = seeder.preview_migration(TpcBenchmark::TpcH, 0.01).await.unwrap();
        
        // TPC-H should have 8 tables
        assert_eq!(table_infos.len(), 8);
        
        // Verify known TPC-H tables exist
        let table_names: Vec<&str> = table_infos.iter().map(|t| t.name.as_str()).collect();
        assert!(table_names.contains(&"customer"));
        assert!(table_names.contains(&"lineitem"));
        assert!(table_names.contains(&"nation"));
        assert!(table_names.contains(&"orders"));
        assert!(table_names.contains(&"part"));
        assert!(table_names.contains(&"partsupp"));
        assert!(table_names.contains(&"region"));
        assert!(table_names.contains(&"supplier"));
    }
    
    #[tokio::test]
    async fn test_simple_migration_preview() {
        let (seeder, _temp_dir) = create_test_seeder().await.unwrap();
        
        // Create test tables without decimals
        seeder.duckdb_conn
            .execute("CREATE TABLE customers (id INTEGER, name VARCHAR, balance DOUBLE)", [])
            .unwrap();
        seeder.duckdb_conn
            .execute("INSERT INTO customers VALUES (1, 'Alice', 100.50), (2, 'Bob', 200.75)", [])
            .unwrap();
            
        seeder.duckdb_conn
            .execute("CREATE TABLE products (id INTEGER, name VARCHAR, price DOUBLE)", [])
            .unwrap();
        seeder.duckdb_conn
            .execute("INSERT INTO products VALUES (1, 'Widget', 9.99), (2, 'Gadget', 19.99)", [])
            .unwrap();
        
        let tables = seeder.discover_tables().unwrap();
        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"customers".to_string()));
        assert!(tables.contains(&"products".to_string()));
        
        // Test migration of a simple table
        seeder.migrate_table("test", "customers").await.unwrap();
    }
    
    #[tokio::test]
    async fn test_get_table_schema() {
        let (seeder, _temp_dir) = create_test_seeder().await.unwrap();
        
        // Create a simple test table without decimals to avoid DuckDB Arrow conversion issues
        seeder.duckdb_conn
            .execute(
                "CREATE TABLE test_table (id INTEGER, name VARCHAR, amount DOUBLE)",
                [],
            )
            .unwrap();
        
        let schema = seeder.get_table_schema("test_table").unwrap();
        assert_eq!(schema.fields().len(), 3);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "name");
        assert_eq!(schema.field(2).name(), "amount");
    }
    
    #[tokio::test]
    #[ignore = "DuckDB has a panic bug with decimal types in Arrow conversion"]
    async fn test_get_table_schema_with_decimal() {
        let (seeder, _temp_dir) = create_test_seeder().await.unwrap();
        
        // Create a test table with decimal - this should use the fallback mechanism
        seeder.duckdb_conn
            .execute(
                "CREATE TABLE test_decimal_table (id INTEGER, price DECIMAL(10,2))",
                [],
            )
            .unwrap();
        
        // This should use the information_schema fallback
        let schema = seeder.get_table_schema("test_decimal_table").unwrap();
        assert_eq!(schema.fields().len(), 2);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "price");
        
        // Check that price is converted to Decimal128
        match schema.field(1).data_type() {
            arrow::datatypes::DataType::Decimal128(p, s) => {
                assert_eq!(*p, 10);
                assert_eq!(*s, 2);
            }
            _ => panic!("Expected Decimal128 type for price field")
        }
    }
    
    #[tokio::test]
    async fn test_get_row_count() {
        let (seeder, _temp_dir) = create_test_seeder().await.unwrap();
        
        // Create and populate a test table
        seeder.duckdb_conn
            .execute("CREATE TABLE test_count (id INTEGER)", [])
            .unwrap();
        seeder.duckdb_conn
            .execute("INSERT INTO test_count VALUES (1), (2), (3)", [])
            .unwrap();
        
        let count = seeder.get_row_count("test_count").unwrap();
        assert_eq!(count, 3);
    }
}

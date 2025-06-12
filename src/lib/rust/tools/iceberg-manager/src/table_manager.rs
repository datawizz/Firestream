use crate::catalog_provider::register_iceberg_catalog;
use crate::error::{Error, Result};
use comfy_table::{Cell, Table as ComfyTable};
use datafusion::prelude::*;
use iceberg::{Catalog, NamespaceIdent, TableIdent, TableCreation};
use iceberg::spec::{Schema, NestedField, Type, PrimitiveType, UnboundPartitionSpec};
use iceberg::io::FileIOBuilder;
use iceberg_catalog_memory::MemoryCatalog;
use iceberg_catalog_rest::{RestCatalog, RestCatalogConfig};
use iceberg_datafusion::IcebergTableProvider;
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for catalog connection
pub enum CatalogConfig {
    Memory {
        warehouse: String,
    },
    Rest {
        uri: String,
        warehouse: String,
        props: HashMap<String, String>,
    },
}

/// High-level table management operations with DataFusion integration
pub struct TableManager {
    catalog: Arc<dyn Catalog>,
    context: SessionContext,
    #[allow(dead_code)]
    catalog_name: String,
}

impl TableManager {
    /// Create a new table manager with the specified catalog configuration
    pub async fn new(config: CatalogConfig) -> Result<Self> {
        let (catalog, catalog_name): (Arc<dyn Catalog>, String) = match config {
            CatalogConfig::Memory { warehouse } => {
                let file_io = FileIOBuilder::new("memory")
                    .build()?;
                let catalog = Arc::new(
                    MemoryCatalog::new(file_io, Some(warehouse))
                );
                (catalog, "memory".to_string())
            }
            CatalogConfig::Rest { uri, warehouse, props } => {
                let config = RestCatalogConfig::builder()
                    .uri(uri)
                    .warehouse(warehouse)
                    .props(props)
                    .build();
                    
                let catalog = Arc::new(
                    RestCatalog::new(config)
                );
                (catalog, "rest".to_string())
            }
        };
        
        let context = SessionContext::new();
        
        // Register the Iceberg catalog with DataFusion
        register_iceberg_catalog(&context, &catalog_name, catalog.clone()).await?;
        
        Ok(Self {
            catalog,
            context,
            catalog_name,
        })
    }
    
    /// List all namespaces in the catalog
    pub async fn list_namespaces(&self) -> Result<Vec<String>> {
        let namespaces = self.catalog
            .list_namespaces(None)
            .await?;
        
        Ok(namespaces
            .into_iter()
            .map(|ns| ns.as_ref().join("."))
            .collect())
    }
    
    /// List tables in a namespace
    pub async fn list_tables(&self, namespace: &str) -> Result<Vec<String>> {
        let ns = self.parse_namespace(namespace)?;
        
        let tables = self.catalog
            .list_tables(&ns)
            .await?;
        
        Ok(tables
            .into_iter()
            .map(|id| id.name().to_string())
            .collect())
    }
    
    /// Create a new namespace
    pub async fn create_namespace(
        &self,
        namespace: &str,
        properties: HashMap<String, String>,
    ) -> Result<()> {
        let ns = self.parse_namespace(namespace)?;
        
        self.catalog
            .create_namespace(&ns, properties)
            .await?;
        
        Ok(())
    }
    
    /// Create a new table
    pub async fn create_table(
        &self,
        namespace: &str,
        table_name: &str,
        schema: Schema,
        partition_spec: Option<UnboundPartitionSpec>,
    ) -> Result<()> {
        let ns = self.parse_namespace(namespace)?;
        
        // Build the table creation in one chain to avoid type mismatch
        let creation = if let Some(spec) = partition_spec {
            TableCreation::builder()
                .name(table_name.to_string())
                .location(format!("/{}/{}", namespace.replace('.', "/"), table_name))
                .schema(schema)
                .partition_spec(spec)
                .build()
        } else {
            TableCreation::builder()
                .name(table_name.to_string())
                .location(format!("/{}/{}", namespace.replace('.', "/"), table_name))
                .schema(schema)
                .build()
        };
        
        self.catalog
            .create_table(&ns, creation)
            .await?;
        
        Ok(())
    }
    
    /// Register a table with DataFusion for SQL queries
    pub async fn register_table(
        &self,
        table_name: &str,
        namespace: &str,
    ) -> Result<()> {
        let ns = self.parse_namespace(namespace)?;
        let table_ident = TableIdent::new(ns, table_name.to_string());
        
        let table = self.catalog
            .load_table(&table_ident)
            .await?;
        
        let df_table = Arc::new(
            IcebergTableProvider::try_new_from_table(table).await?
        );
        
        self.context.register_table(table_name, df_table)?;
        
        Ok(())
    }
    
    /// Execute a SQL query
    pub async fn sql(&self, query: &str) -> Result<DataFrame> {
        self.context
            .sql(query)
            .await
            .map_err(|e| Error::DataFusionError(e.to_string()))
    }
    
    /// Get table schema
    pub async fn get_table_schema(
        &self,
        namespace: &str,
        table_name: &str,
    ) -> Result<TableSchema> {
        let ns = self.parse_namespace(namespace)?;
        let table_ident = TableIdent::new(ns, table_name.to_string());
        
        let table = self.catalog
            .load_table(&table_ident)
            .await?;
        
        let metadata = table.metadata();
        let schema = metadata.current_schema();
        
        Ok(TableSchema::from_iceberg_schema(schema))
    }
    
    /// Preview table data using SQL
    pub async fn preview_table(
        &self,
        namespace: &str,
        table_name: &str,
        limit: Option<usize>,
    ) -> Result<TablePreview> {
        // First register the table
        self.register_table(table_name, namespace).await?;
        
        // Build SQL query
        let query = if let Some(limit) = limit {
            format!("SELECT * FROM {} LIMIT {}", table_name, limit)
        } else {
            format!("SELECT * FROM {}", table_name)
        };
        
        // Execute query
        let df = self.sql(&query).await?;
        let batches = df.collect().await?;
        
        let row_count = batches.iter().map(|b| b.num_rows()).sum();
        
        Ok(TablePreview {
            schema: self.get_table_schema(namespace, table_name).await?,
            row_count,
            batches,
        })
    }
    
    /// Get table statistics
    pub async fn get_table_stats(
        &self,
        namespace: &str,
        table_name: &str,
    ) -> Result<TableStats> {
        let ns = self.parse_namespace(namespace)?;
        let table_ident = TableIdent::new(ns, table_name.to_string());
        
        let table = self.catalog
            .load_table(&table_ident)
            .await?;
        
        let metadata = table.metadata();
        
        Ok(TableStats {
            location: metadata.location().to_string(),
            current_snapshot_id: metadata.current_snapshot().map(|s| s.snapshot_id()),
            snapshot_count: metadata.snapshots().count(),
            properties: metadata.properties().clone(),
        })
    }
    
    /// Drop a table
    pub async fn drop_table(&self, namespace: &str, table_name: &str) -> Result<()> {
        let ns = self.parse_namespace(namespace)?;
        let table_ident = TableIdent::new(ns, table_name.to_string());
        
        self.catalog
            .drop_table(&table_ident)
            .await?;
        
        Ok(())
    }
    
    /// Parse namespace string into NamespaceIdent object
    fn parse_namespace(&self, namespace: &str) -> Result<NamespaceIdent> {
        NamespaceIdent::from_strs(namespace.split('.'))
            .map_err(|e| Error::InvalidOperation(format!("Invalid namespace: {}", e)))
    }
}

/// Table schema information
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub fields: Vec<TableField>,
}

#[derive(Debug, Clone)]
pub struct TableField {
    pub id: i32,
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub doc: Option<String>,
}

impl TableSchema {
    fn from_iceberg_schema(schema: &Schema) -> Self {
        // Access fields through the schema's struct type
        let struct_type = schema.as_struct();
        let fields = struct_type.fields()
            .iter()
            .map(|field| TableField {
                id: field.id,
                name: field.name.clone(),
                data_type: format_type(&field.field_type),
                required: field.required,
                doc: field.doc.clone(),
            })
            .collect();
        
        Self { fields }
    }
    
    /// Display schema as a formatted table
    pub fn display(&self) -> String {
        let mut table = ComfyTable::new();
        table.set_header(vec!["ID", "Name", "Type", "Required", "Comment"]);
        
        for field in &self.fields {
            table.add_row(vec![
                Cell::new(field.id),
                Cell::new(&field.name),
                Cell::new(&field.data_type),
                Cell::new(if field.required { "Yes" } else { "No" }),
                Cell::new(field.doc.as_deref().unwrap_or("")),
            ]);
        }
        
        table.to_string()
    }
}

/// Table preview data
pub struct TablePreview {
    pub schema: TableSchema,
    pub row_count: usize,
    pub batches: Vec<datafusion::arrow::array::RecordBatch>,
}

impl TablePreview {
    /// Display preview as a formatted table
    pub fn display(&self) -> Result<String> {
        if self.batches.is_empty() {
            return Ok("No data available".to_string());
        }
        
        let formatted = datafusion::arrow::util::pretty::pretty_format_batches(&self.batches)?;
        Ok(formatted.to_string())
    }
}

/// Table statistics
#[derive(Debug)]
pub struct TableStats {
    pub location: String,
    pub current_snapshot_id: Option<i64>,
    pub snapshot_count: usize,
    pub properties: HashMap<String, String>,
}

impl TableStats {
    /// Display stats as a formatted table
    pub fn display(&self) -> String {
        let mut table = ComfyTable::new();
        table.set_header(vec!["Property", "Value"]);
        
        table.add_row(vec!["Location", &self.location]);
        table.add_row(vec![
            "Current Snapshot ID",
            &self.current_snapshot_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "None".to_string()),
        ]);
        table.add_row(vec!["Snapshot Count", &self.snapshot_count.to_string()]);
        
        for (key, value) in &self.properties {
            table.add_row(vec![key, value]);
        }
        
        table.to_string()
    }
}

/// Format an Iceberg type to a string representation
fn format_type(t: &Type) -> String {
    match t {
        Type::Primitive(p) => match p {
            PrimitiveType::Boolean => "boolean".to_string(),
            PrimitiveType::Int => "int".to_string(),
            PrimitiveType::Long => "long".to_string(),
            PrimitiveType::Float => "float".to_string(),
            PrimitiveType::Double => "double".to_string(),
            PrimitiveType::Decimal { precision, scale } => {
                format!("decimal({}, {})", precision, scale)
            }
            PrimitiveType::Date => "date".to_string(),
            PrimitiveType::Time => "time".to_string(),
            PrimitiveType::Timestamp => "timestamp".to_string(),
            PrimitiveType::Timestamptz => "timestamptz".to_string(),
            PrimitiveType::TimestampNs => "timestamp_ns".to_string(),
            PrimitiveType::TimestamptzNs => "timestamptz_ns".to_string(),
            PrimitiveType::String => "string".to_string(),
            PrimitiveType::Uuid => "uuid".to_string(),
            PrimitiveType::Fixed(len) => format!("fixed[{}]", len),
            PrimitiveType::Binary => "binary".to_string(),
        },
        Type::Struct(_) => "struct".to_string(),
        Type::List(_) => "list".to_string(),
        Type::Map { .. } => "map".to_string(),
    }
}

/// Create a simple test schema using official iceberg API
pub fn create_test_schema() -> Result<Schema> {
    let schema = Schema::builder()
        .with_schema_id(1)
        .with_fields(vec![
            NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)).into(),
            NestedField::required(2, "name", Type::Primitive(PrimitiveType::String)).into(),
            NestedField::optional(3, "email", Type::Primitive(PrimitiveType::String)).into(),
            NestedField::required(4, "created_at", Type::Primitive(PrimitiveType::Timestamp)).into(),
        ])
        .build()
        .map_err(|e| Error::InvalidOperation(format!("Failed to build schema: {}", e)))?;
    
    Ok(schema)
}

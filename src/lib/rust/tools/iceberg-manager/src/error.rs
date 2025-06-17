use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Iceberg error: {0}")]
    Iceberg(#[from] iceberg::Error),
    
    #[error("Catalog error: {0}")]
    CatalogError(String),
    
    #[error("Table creation error: {0}")]
    TableCreationError(String),
    
    #[error("Table load error: {0}")]
    TableLoadError(String),
    
    #[error("DataFusion error: {0}")]
    DataFusionError(String),
    
    #[error("DataFusion error")]
    DataFusion(#[from] datafusion::error::DataFusionError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Table not found: {0}")]
    TableNotFound(String),
    
    #[error("Namespace not found: {0}")]
    NamespaceNotFound(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Arrow error: {0}")]
    ArrowError(String),
    
    #[error("Arrow error")]
    Arrow(#[from] arrow::error::ArrowError),
    
    #[error("DuckDB error: {0}")]
    DuckDB(#[from] duckdb::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

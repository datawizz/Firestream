use serde::{Deserialize, Serialize};

/// Supported file formats for Iceberg tables
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FileFormat {
    Parquet,
    Avro,
    Orc,
}

impl Default for FileFormat {
    fn default() -> Self {
        FileFormat::Parquet
    }
}

/// Configuration options for table creation
#[derive(Debug, Clone, Default)]
pub struct TableOptions {
    pub file_format: FileFormat,
    pub target_file_size: Option<usize>,
    pub compression: Option<CompressionType>,
}

/// Supported compression types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Snappy,
    Gzip,
    Lz4,
    Zstd,
}

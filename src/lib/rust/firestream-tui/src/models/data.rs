use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaTable {
    pub name: String,
    pub database: String,
    pub location: String,
    pub format: String,
    #[serde(rename = "partitionColumns")]
    pub partition_columns: Vec<String>,
    #[serde(rename = "numFiles")]
    pub num_files: u64,
    #[serde(rename = "sizeInBytes")]
    pub size_in_bytes: u64,
    #[serde(rename = "lastModified")]
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    #[serde(rename = "tableName")]
    pub table_name: String,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: String,
    pub nullable: bool,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LakeFSBranch {
    pub name: String,
    pub repository: String,
    #[serde(rename = "commitId")]
    pub commit_id: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    pub creator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Bucket {
    pub name: String,
    #[serde(rename = "creationDate")]
    pub creation_date: DateTime<Utc>,
    #[serde(rename = "sizeInBytes")]
    pub size_in_bytes: u64,
    #[serde(rename = "objectCount")]
    pub object_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Object {
    pub key: String,
    pub size: u64,
    #[serde(rename = "lastModified")]
    pub last_modified: DateTime<Utc>,
    pub etag: String,
    #[serde(rename = "storageClass")]
    pub storage_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3ObjectList {
    pub objects: Vec<S3Object>,
    #[serde(rename = "isTruncated")]
    pub is_truncated: bool,
    #[serde(rename = "nextContinuationToken")]
    pub next_continuation_token: Option<String>,
}

// Iceberg-specific types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcebergCatalog {
    pub name: String,
    pub catalog_type: IcebergCatalogType,
    pub warehouse: String,
    pub namespaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IcebergCatalogType {
    Memory,
    Rest,
    FileSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcebergNamespace {
    pub name: String,
    pub catalog: String,
    pub tables: Vec<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcebergTable {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub catalog: String,
    pub location: String,
    pub current_snapshot_id: Option<i64>,
    pub schema: IcebergSchema,
    pub partition_spec: Option<Vec<PartitionField>>,
    pub properties: HashMap<String, String>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcebergSchema {
    pub schema_id: i32,
    pub fields: Vec<IcebergField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcebergField {
    pub id: i32,
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionField {
    pub source_id: i32,
    pub field_id: i32,
    pub name: String,
    pub transform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub storage_type: StorageType,
    pub credentials: StorageCredentials,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    LocalFileSystem,
    S3,
    GoogleCloudStorage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageCredentials {
    None, // For local filesystem
    S3 {
        access_key_id: String,
        secret_access_key: String,
        region: String,
        endpoint: Option<String>, // For S3-compatible stores
    },
    Gcs {
        service_account_key: String,
        project_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcebergQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

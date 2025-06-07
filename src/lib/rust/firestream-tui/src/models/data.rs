use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

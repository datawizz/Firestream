use iceberg_manager::{TableManager, CatalogConfig as IcebergCatalogConfig, Schema, NestedField, Type, PrimitiveType};
use crate::models::{StorageConfig, StorageType, StorageCredentials, IcebergCatalog, IcebergCatalogType};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IcebergServiceError {
    #[error("Iceberg manager error: {0}")]
    ManagerError(#[from] iceberg_manager::Error),
    
    #[error("Storage configuration error: {0}")]
    StorageConfigError(String),
    
    #[error("Catalog not found: {0}")]
    CatalogNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

type Result<T> = std::result::Result<T, IcebergServiceError>;

pub struct IcebergService {
    managers: Arc<Mutex<HashMap<String, Arc<TableManager>>>>,
    catalog_configs: Arc<Mutex<HashMap<String, StorageConfig>>>,
}

impl IcebergService {
    pub fn new() -> Self {
        Self {
            managers: Arc::new(Mutex::new(HashMap::new())),
            catalog_configs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Initialize default catalogs (like local filesystem)
    pub async fn init_default_catalogs(&self) -> Result<()> {
        // Create default local catalog
        let local_config = StorageConfig {
            storage_type: StorageType::LocalFileSystem,
            credentials: StorageCredentials::None,
        };
        
        self.create_catalog("local", local_config).await?;
        Ok(())
    }
    
    /// Create a new catalog with the given configuration
    pub async fn create_catalog(&self, name: &str, config: StorageConfig) -> Result<IcebergCatalog> {
        let iceberg_config = self.build_catalog_config(&config)?;
        let manager = Arc::new(TableManager::new(iceberg_config).await?);
        
        let mut managers = self.managers.lock().await;
        managers.insert(name.to_string(), manager.clone());
        
        let mut configs = self.catalog_configs.lock().await;
        configs.insert(name.to_string(), config.clone());
        
        // Return catalog info
        Ok(IcebergCatalog {
            name: name.to_string(),
            catalog_type: match config.storage_type {
                StorageType::LocalFileSystem => IcebergCatalogType::Memory,
                StorageType::S3 | StorageType::GoogleCloudStorage => IcebergCatalogType::Rest,
            },
            warehouse: self.get_warehouse_path(&config),
            namespaces: vec![], // Will be populated when listing
        })
    }
    
    /// Get or create a manager for a catalog
    pub async fn get_manager(&self, catalog_name: &str) -> Result<Arc<TableManager>> {
        let managers = self.managers.lock().await;
        
        if let Some(manager) = managers.get(catalog_name) {
            Ok(manager.clone())
        } else {
            Err(IcebergServiceError::CatalogNotFound(catalog_name.to_string()))
        }
    }
    
    /// List all catalogs
    pub async fn list_catalogs(&self) -> Result<Vec<IcebergCatalog>> {
        let managers = self.managers.lock().await;
        let configs = self.catalog_configs.lock().await;
        
        let mut catalogs = Vec::new();
        
        for (name, _) in managers.iter() {
            if let Some(config) = configs.get(name) {
                let manager = managers.get(name).unwrap();
                
                // Get namespaces for this catalog
                let namespaces = match manager.list_namespaces().await {
                    Ok(ns) => ns,
                    Err(_) => vec![],
                };
                
                catalogs.push(IcebergCatalog {
                    name: name.clone(),
                    catalog_type: match config.storage_type {
                        StorageType::LocalFileSystem => IcebergCatalogType::Memory,
                        StorageType::S3 | StorageType::GoogleCloudStorage => IcebergCatalogType::Rest,
                    },
                    warehouse: self.get_warehouse_path(config),
                    namespaces,
                });
            }
        }
        
        Ok(catalogs)
    }
    
    /// Build iceberg-manager catalog config from our storage config
    fn build_catalog_config(&self, config: &StorageConfig) -> Result<IcebergCatalogConfig> {
        match &config.storage_type {
            StorageType::LocalFileSystem => {
                // Use LOCAL_DATA_DIRECTORY environment variable if set, otherwise default
                let warehouse = std::env::var("LOCAL_DATA_DIRECTORY")
                    .map(|dir| format!("{}/iceberg-warehouse", dir))
                    .unwrap_or_else(|_| "/tmp/iceberg-warehouse".to_string());
                
                Ok(IcebergCatalogConfig::Memory {
                    warehouse,
                })
            }
            StorageType::S3 => {
                match &config.credentials {
                    StorageCredentials::S3 { region, endpoint, .. } => {
                        let mut props = HashMap::new();
                        
                        // Configure S3 properties
                        props.insert("region".to_string(), region.clone());
                        if let Some(endpoint) = endpoint {
                            props.insert("endpoint".to_string(), endpoint.clone());
                        }
                        
                        // TODO: Add authentication properties when REST catalog is properly configured
                        
                        Ok(IcebergCatalogConfig::Rest {
                            uri: endpoint.clone().unwrap_or_else(|| "http://localhost:8181".to_string()),
                            warehouse: "s3://iceberg-warehouse".to_string(),
                            props,
                        })
                    }
                    _ => Err(IcebergServiceError::InvalidConfig(
                        "S3 storage requires S3 credentials".to_string()
                    )),
                }
            }
            StorageType::GoogleCloudStorage => {
                match &config.credentials {
                    StorageCredentials::Gcs { project_id, .. } => {
                        let mut props = HashMap::new();
                        props.insert("project_id".to_string(), project_id.clone());
                        
                        Ok(IcebergCatalogConfig::Rest {
                            uri: "http://localhost:8181".to_string(), // TODO: Configure GCS endpoint
                            warehouse: "gs://iceberg-warehouse".to_string(),
                            props,
                        })
                    }
                    _ => Err(IcebergServiceError::InvalidConfig(
                        "GCS storage requires GCS credentials".to_string()
                    )),
                }
            }
        }
    }
    
    fn get_warehouse_path(&self, config: &StorageConfig) -> String {
        match &config.storage_type {
            StorageType::LocalFileSystem => {
                // Use LOCAL_DATA_DIRECTORY environment variable if set, otherwise default
                std::env::var("LOCAL_DATA_DIRECTORY")
                    .map(|dir| format!("{}/iceberg-warehouse", dir))
                    .unwrap_or_else(|_| "/tmp/iceberg-warehouse".to_string())
            },
            StorageType::S3 => "s3://iceberg-warehouse".to_string(),
            StorageType::GoogleCloudStorage => "gs://iceberg-warehouse".to_string(),
        }
    }
    
    /// Helper function to create a simple table schema
    pub fn create_simple_schema() -> Result<Schema> {
        let schema = Schema::builder()
            .with_schema_id(1)
            .with_fields(vec![
                NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)).into(),
                NestedField::required(2, "name", Type::Primitive(PrimitiveType::String)).into(),
                NestedField::optional(3, "value", Type::Primitive(PrimitiveType::Double)).into(),
                NestedField::required(4, "timestamp", Type::Primitive(PrimitiveType::Timestamp)).into(),
            ])
            .build()
            .map_err(|e| IcebergServiceError::InvalidConfig(format!("Failed to build schema: {}", e)))?;
        
        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_local_catalog() {
        let service = IcebergService::new();
        
        let config = StorageConfig {
            storage_type: StorageType::LocalFileSystem,
            credentials: StorageCredentials::None,
        };
        
        let catalog = service.create_catalog("test", config).await.unwrap();
        assert_eq!(catalog.name, "test");
        assert!(matches!(catalog.catalog_type, IcebergCatalogType::Memory));
    }
}

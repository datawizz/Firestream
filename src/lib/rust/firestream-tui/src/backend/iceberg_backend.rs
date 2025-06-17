//! Iceberg backend implementation that provides real data operations.
//!
//! This backend is automatically used when the `LOCAL_DATA_DIRECTORY` environment variable
//! is set. It provides:
//! - Real Iceberg catalog, namespace, and table browsing
//! - Actual table schema information
//! - SQL query execution through DataFusion
//! - Table data preview functionality
//!
//! Non-Iceberg resources (deployments, templates, etc.) still use mock data
//! as they require a full Firestream API server.

use super::{ApiResult, ApiError, FirestreamBackend, BoxFuture};
use crate::models::*;
use crate::services::IcebergService;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct IcebergBackend {
    iceberg_service: Arc<Mutex<IcebergService>>,
    // Keep mock implementations for non-iceberg resources
    mock_deployments: Vec<Deployment>,
    mock_templates: Vec<Template>,
    mock_nodes: Vec<Node>,
    mock_secrets: Vec<SecretInfo>,
}

impl IcebergBackend {
    pub async fn new() -> Self {
        let iceberg_service = Arc::new(Mutex::new(IcebergService::new()));
        
        // Initialize default catalogs
        if let Err(e) = iceberg_service.lock().await.init_default_catalogs().await {
            eprintln!("Warning: Failed to initialize default catalogs: {}", e);
        }
        
        Self {
            iceberg_service,
            // Initialize with some mock data for non-iceberg resources
            mock_deployments: vec![
                Deployment {
                    id: "dep-1".to_string(),
                    name: "etl-pipeline".to_string(),
                    template_id: "pyspark-base".to_string(),
                    namespace: "default".to_string(),
                    status: DeploymentStatus::Running,
                    replicas: ReplicaStatus {
                        desired: 3,
                        ready: 3,
                        available: 3,
                    },
                    created_at: Utc::now() - chrono::Duration::hours(2),
                    updated_at: Utc::now() - chrono::Duration::minutes(5),
                },
            ],
            mock_templates: vec![
                Template {
                    id: "pyspark-base".to_string(),
                    name: "PySpark Application".to_string(),
                    template_type: TemplateType::PySpark,
                    version: "1.0.0".to_string(),
                    description: Some("Full-featured PySpark application template".to_string()),
                    config: FirestreamConfig {
                        app: AppConfig {
                            name: "pyspark-app".to_string(),
                            version: "1.0.0".to_string(),
                            app_type: "pyspark".to_string(),
                        },
                        resources: ResourceConfig {
                            cpu: "4".to_string(),
                            memory: "8Gi".to_string(),
                            gpu: Some(false),
                            gpu_type: None,
                        },
                        kafka: None,
                        input_schema: None,
                        output_structure: None,
                    },
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ],
            mock_nodes: vec![
                Node {
                    id: "node-1".to_string(),
                    name: "worker-01".to_string(),
                    provider: NodeProvider::Local,
                    instance_type: "standard-8".to_string(),
                    status: NodeStatus::Ready,
                    resources: NodeResources {
                        cpu: 8,
                        memory: "32Gi".to_string(),
                        gpu: None,
                    },
                    labels: HashMap::new(),
                    spot: false,
                    created_at: Utc::now() - chrono::Duration::days(5),
                },
            ],
            mock_secrets: vec![
                SecretInfo {
                    name: "api-keys".to_string(),
                    namespace: "default".to_string(),
                    secret_type: SecretType::Opaque,
                    keys: vec!["api_key".to_string(), "api_secret".to_string()],
                    created_at: Utc::now() - chrono::Duration::days(10),
                    updated_at: Utc::now() - chrono::Duration::days(2),
                },
            ],
        }
    }
}

impl FirestreamBackend for IcebergBackend {
    // Keep mock implementations for non-iceberg resources
    fn list_templates(&self) -> BoxFuture<'_, ApiResult<Vec<Template>>> {
        Box::pin(async move {
            Ok(self.mock_templates.clone())
        })
    }

    fn get_template(&self, id: &str) -> BoxFuture<'_, ApiResult<Template>> {
        let id = id.to_string();
        Box::pin(async move {
            self.mock_templates.iter()
                .find(|t| t.id == id)
                .cloned()
                .ok_or(ApiError::NotFound)
        })
    }

    fn list_deployments(&self) -> BoxFuture<'_, ApiResult<Vec<Deployment>>> {
        Box::pin(async move {
            Ok(self.mock_deployments.clone())
        })
    }

    fn get_deployment(&self, id: &str) -> BoxFuture<'_, ApiResult<DeploymentDetail>> {
        let id = id.to_string();
        Box::pin(async move {
            let deployment = self.mock_deployments.iter()
                .find(|d| d.id == id)
                .ok_or(ApiError::NotFound)?;
            
            Ok(DeploymentDetail {
                deployment: deployment.clone(),
                pods: vec![],
                services: vec![],
                ingress: vec![],
            })
        })
    }

    fn create_deployment(&self, template_id: &str, name: &str, _payload: serde_json::Value) -> BoxFuture<'_, ApiResult<Deployment>> {
        let template_id = template_id.to_string();
        let name = name.to_string();
        Box::pin(async move {
            Ok(Deployment {
                id: format!("dep-{}", uuid::Uuid::new_v4()),
                name,
                template_id,
                namespace: "default".to_string(),
                status: DeploymentStatus::Pending,
                replicas: ReplicaStatus {
                    desired: 1,
                    ready: 0,
                    available: 0,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        })
    }

    fn scale_deployment(&self, _id: &str, _replicas: u32) -> BoxFuture<'_, ApiResult<Deployment>> {
        Box::pin(async move {
            Err(ApiError::Unknown("Not implemented".to_string()))
        })
    }

    fn delete_deployment(&self, _id: &str) -> BoxFuture<'_, ApiResult<()>> {
        Box::pin(async move {
            Ok(())
        })
    }

    fn get_deployment_logs(&self, _id: &str, _tail: Option<u32>) -> BoxFuture<'_, ApiResult<String>> {
        Box::pin(async move {
            Ok("Mock deployment logs...".to_string())
        })
    }

    fn get_cluster_status(&self) -> BoxFuture<'_, ApiResult<ClusterStatus>> {
        Box::pin(async move {
            let mut services = HashMap::new();
            services.insert("iceberg".to_string(), ServiceStatus {
                status: HealthStatus::Healthy,
                replicas: "1/1".to_string(),
                endpoint: None,
            });

            Ok(ClusterStatus {
                name: "local-iceberg".to_string(),
                provider: ClusterProvider::K3d,
                status: HealthStatus::Healthy,
                nodes: NodeSummary {
                    total: 1,
                    ready: 1,
                    gpu: 0,
                },
                resources: ClusterResourceUtilization {
                    cpu_usage: 10.0,
                    memory_usage: 15.0,
                    gpu_usage: None,
                },
                services,
            })
        })
    }

    fn list_services(&self) -> BoxFuture<'_, ApiResult<Vec<Service>>> {
        Box::pin(async move {
            Ok(vec![])
        })
    }

    fn list_nodes(&self) -> BoxFuture<'_, ApiResult<Vec<Node>>> {
        Box::pin(async move {
            Ok(self.mock_nodes.clone())
        })
    }

    fn get_node(&self, id: &str) -> BoxFuture<'_, ApiResult<NodeDetail>> {
        let id = id.to_string();
        Box::pin(async move {
            let node = self.mock_nodes.iter()
                .find(|n| n.id == id)
                .ok_or(ApiError::NotFound)?;
            
            Ok(NodeDetail {
                node: node.clone(),
                pods: vec![],
                utilization: node::ResourceUtilization {
                    cpu: 10.0,
                    memory: 15.0,
                    gpu: None,
                },
            })
        })
    }

    fn provision_node(&self, _provider: &str, _instance_type: &str) -> BoxFuture<'_, ApiResult<String>> {
        Box::pin(async move {
            Ok(format!("node-{}", uuid::Uuid::new_v4()))
        })
    }

    // Mock implementations for non-iceberg data operations
    fn list_delta_tables(&self) -> BoxFuture<'_, ApiResult<Vec<DeltaTable>>> {
        Box::pin(async move {
            Ok(vec![])
        })
    }

    fn get_table_schema(&self, _table_name: &str) -> BoxFuture<'_, ApiResult<TableSchema>> {
        Box::pin(async move {
            Err(ApiError::NotFound)
        })
    }

    fn list_lakefs_branches(&self) -> BoxFuture<'_, ApiResult<Vec<LakeFSBranch>>> {
        Box::pin(async move {
            Ok(vec![])
        })
    }

    fn list_s3_buckets(&self) -> BoxFuture<'_, ApiResult<Vec<S3Bucket>>> {
        Box::pin(async move {
            Ok(vec![])
        })
    }

    fn start_build(&self, _deployment_id: &str) -> BoxFuture<'_, ApiResult<BuildStatus>> {
        Box::pin(async move {
            Err(ApiError::Unknown("Not implemented".to_string()))
        })
    }

    fn get_build_status(&self, _build_id: &str) -> BoxFuture<'_, ApiResult<BuildStatus>> {
        Box::pin(async move {
            Err(ApiError::NotFound)
        })
    }

    fn get_build_logs(&self, _build_id: &str) -> BoxFuture<'_, ApiResult<String>> {
        Box::pin(async move {
            Ok("No build logs available".to_string())
        })
    }

    fn list_secrets(&self) -> BoxFuture<'_, ApiResult<Vec<SecretInfo>>> {
        Box::pin(async move {
            Ok(self.mock_secrets.clone())
        })
    }

    fn get_secret(&self, id: &str) -> BoxFuture<'_, ApiResult<SecretInfo>> {
        let id = id.to_string();
        Box::pin(async move {
            self.mock_secrets.iter()
                .find(|s| s.name == id)
                .cloned()
                .ok_or(ApiError::NotFound)
        })
    }

    fn create_secret(&self, name: &str, data: HashMap<String, String>) -> BoxFuture<'_, ApiResult<SecretInfo>> {
        let name = name.to_string();
        Box::pin(async move {
            Ok(SecretInfo {
                name,
                namespace: "default".to_string(),
                secret_type: SecretType::Opaque,
                keys: data.keys().cloned().collect(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        })
    }

    fn delete_secret(&self, _name: &str) -> BoxFuture<'_, ApiResult<()>> {
        Box::pin(async move {
            Ok(())
        })
    }

    // Real Iceberg implementations
    fn list_iceberg_catalogs(&self) -> BoxFuture<'_, ApiResult<Vec<IcebergCatalog>>> {
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            service.list_catalogs().await
                .map_err(|e| ApiError::Unknown(e.to_string()))
        })
    }

    fn get_iceberg_catalog(&self, name: &str) -> BoxFuture<'_, ApiResult<IcebergCatalog>> {
        let name = name.to_string();
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            let catalogs = service.list_catalogs().await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            catalogs.into_iter()
                .find(|c| c.name == name)
                .ok_or(ApiError::NotFound)
        })
    }

    fn create_iceberg_catalog(&self, config: &StorageConfig) -> BoxFuture<'_, ApiResult<IcebergCatalog>> {
        let config = config.clone();
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            service.create_catalog("new_catalog", config).await
                .map_err(|e| ApiError::Unknown(e.to_string()))
        })
    }

    fn list_iceberg_namespaces(&self, catalog: &str) -> BoxFuture<'_, ApiResult<Vec<IcebergNamespace>>> {
        let catalog = catalog.to_string();
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            let manager = service.get_manager(&catalog).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            let namespaces = manager.list_namespaces().await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            // Convert namespace strings to IcebergNamespace objects
            let mut result = Vec::new();
            for ns_name in namespaces {
                // Get tables for this namespace
                let tables = manager.list_tables(&ns_name).await
                    .unwrap_or_default();
                
                result.push(IcebergNamespace {
                    name: ns_name,
                    catalog: catalog.clone(),
                    tables,
                    properties: HashMap::new(),
                });
            }
            
            Ok(result)
        })
    }

    fn create_iceberg_namespace(&self, catalog: &str, namespace: &str, properties: HashMap<String, String>) -> BoxFuture<'_, ApiResult<IcebergNamespace>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            let manager = service.get_manager(&catalog).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            manager.create_namespace(&namespace, properties.clone()).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            Ok(IcebergNamespace {
                name: namespace,
                catalog,
                tables: vec![],
                properties,
            })
        })
    }

    fn list_iceberg_tables(&self, catalog: &str, namespace: &str) -> BoxFuture<'_, ApiResult<Vec<IcebergTable>>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            let manager = service.get_manager(&catalog).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            let table_names = manager.list_tables(&namespace).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            let mut tables = Vec::new();
            for table_name in table_names {
                // Get table details
                match manager.get_table_stats(&namespace, &table_name).await {
                    Ok(stats) => {
                        // Get schema
                        if let Ok(schema) = manager.get_table_schema(&namespace, &table_name).await {
                            let iceberg_schema = IcebergSchema {
                                schema_id: 1,
                                fields: schema.fields.into_iter().map(|f| IcebergField {
                                    id: f.id,
                                    name: f.name,
                                    field_type: f.data_type,
                                    required: f.required,
                                    doc: f.doc,
                                }).collect(),
                            };
                            
                            tables.push(IcebergTable {
                                id: format!("{}.{}.{}", catalog, namespace, table_name),
                                name: table_name,
                                namespace: namespace.clone(),
                                catalog: catalog.clone(),
                                location: stats.location,
                                current_snapshot_id: stats.current_snapshot_id,
                                schema: iceberg_schema,
                                partition_spec: None, // TODO: Get from table metadata
                                properties: stats.properties,
                                last_modified: Utc::now(), // TODO: Get from metadata
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get stats for table {}.{}: {}", namespace, table_name, e);
                    }
                }
            }
            
            Ok(tables)
        })
    }

    fn get_iceberg_table(&self, catalog: &str, namespace: &str, table: &str) -> BoxFuture<'_, ApiResult<IcebergTable>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        let table = table.to_string();
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            let manager = service.get_manager(&catalog).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            let stats = manager.get_table_stats(&namespace, &table).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            let schema = manager.get_table_schema(&namespace, &table).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            let iceberg_schema = IcebergSchema {
                schema_id: 1,
                fields: schema.fields.into_iter().map(|f| IcebergField {
                    id: f.id,
                    name: f.name,
                    field_type: f.data_type,
                    required: f.required,
                    doc: f.doc,
                }).collect(),
            };
            
            Ok(IcebergTable {
                id: format!("{}.{}.{}", catalog, namespace, table),
                name: table,
                namespace,
                catalog,
                location: stats.location,
                current_snapshot_id: stats.current_snapshot_id,
                schema: iceberg_schema,
                partition_spec: None,
                properties: stats.properties,
                last_modified: Utc::now(),
            })
        })
    }

    fn create_iceberg_table(&self, catalog: &str, namespace: &str, table: &str, _schema: IcebergSchema, _partition_spec: Option<Vec<PartitionField>>) -> BoxFuture<'_, ApiResult<IcebergTable>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        let table = table.to_string();
        Box::pin(async move {
            // Convert our schema to iceberg-manager schema
            let service = self.iceberg_service.lock().await;
            let manager = service.get_manager(&catalog).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            // Use the helper function to create a simple schema for now
            let iceberg_schema = IcebergService::create_simple_schema()
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            manager.create_table(&namespace, &table, iceberg_schema, None).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            // Return the created table
            self.get_iceberg_table(&catalog, &namespace, &table).await
        })
    }

    fn drop_iceberg_table(&self, catalog: &str, namespace: &str, table: &str) -> BoxFuture<'_, ApiResult<()>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        let table = table.to_string();
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            let manager = service.get_manager(&catalog).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            manager.drop_table(&namespace, &table).await
                .map_err(|e| ApiError::Unknown(e.to_string()))
        })
    }

    fn query_iceberg_table(&self, catalog: &str, namespace: &str, table: &str, sql: &str) -> BoxFuture<'_, ApiResult<IcebergQueryResult>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        let table = table.to_string();
        let sql = sql.to_string();
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            let manager = service.get_manager(&catalog).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            // Register the table first
            manager.register_table(&table, &namespace).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            // Execute the query
            let df = manager.sql(&sql).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            let batches = df.collect().await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            // Convert to our query result format
            if batches.is_empty() {
                return Ok(IcebergQueryResult {
                    columns: vec![],
                    rows: vec![],
                    row_count: 0,
                });
            }
            
            let schema = batches[0].schema();
            let columns: Vec<String> = schema.fields()
                .iter()
                .map(|f| f.name().clone())
                .collect();
            
            let mut rows = Vec::new();
            for batch in &batches {
                for row_idx in 0..batch.num_rows() {
                    let mut row = Vec::new();
                    for col_idx in 0..batch.num_columns() {
                        let column = batch.column(col_idx);
                        // Convert Arrow value to JSON
                        // This is simplified - you'd need proper type handling
                        let value = if column.is_null(row_idx) {
                            serde_json::Value::Null
                        } else {
                            // Basic string representation for now
                            serde_json::json!(format!("{:?}", column))
                        };
                        row.push(value);
                    }
                    rows.push(row);
                }
            }
            
            let row_count = rows.len();
            Ok(IcebergQueryResult {
                columns,
                rows,
                row_count,
            })
        })
    }

    fn preview_iceberg_table(&self, catalog: &str, namespace: &str, table: &str, limit: usize) -> BoxFuture<'_, ApiResult<IcebergQueryResult>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        let table = table.to_string();
        Box::pin(async move {
            let service = self.iceberg_service.lock().await;
            let manager = service.get_manager(&catalog).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            // Register the table
            manager.register_table(&table, &namespace).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            // Preview with limit
            let preview = manager.preview_table(&namespace, &table, Some(limit)).await
                .map_err(|e| ApiError::Unknown(e.to_string()))?;
            
            // Convert preview to query result
            if preview.batches.is_empty() {
                return Ok(IcebergQueryResult {
                    columns: vec![],
                    rows: vec![],
                    row_count: 0,
                });
            }
            
            let schema = preview.batches[0].schema();
            let columns: Vec<String> = schema.fields()
                .iter()
                .map(|f| f.name().clone())
                .collect();
            
            let mut rows = Vec::new();
            for batch in &preview.batches {
                for row_idx in 0..batch.num_rows() {
                    let mut row = Vec::new();
                    for col_idx in 0..batch.num_columns() {
                        let column = batch.column(col_idx);
                        // Convert based on data type
                        use arrow_array::{Int64Array, StringArray, Float64Array};
                        use arrow::datatypes::DataType;
                        let value = if column.is_null(row_idx) {
                            serde_json::Value::Null
                        } else {
                            match column.data_type() {
                                DataType::Int64 => {
                                    let array = column.as_any().downcast_ref::<Int64Array>().unwrap();
                                    serde_json::json!(array.value(row_idx))
                                }
                                DataType::Utf8 => {
                                    let array = column.as_any().downcast_ref::<StringArray>().unwrap();
                                    serde_json::json!(array.value(row_idx))
                                }
                                DataType::Float64 => {
                                    let array = column.as_any().downcast_ref::<Float64Array>().unwrap();
                                    serde_json::json!(array.value(row_idx))
                                }
                                _ => {
                                    // Fallback to string representation
                                    serde_json::json!("unsupported type")
                                }
                            }
                        };
                        row.push(value);
                    }
                    rows.push(row);
                }
            }
            
            let row_count = rows.len();
            Ok(IcebergQueryResult {
                columns,
                rows,
                row_count,
            })
        })
    }
}

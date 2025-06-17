use super::{ApiResult, ApiError, FirestreamBackend, BoxFuture};
use crate::models::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct MockClient;

impl MockClient {
    pub fn new() -> Self {
        Self
    }
}

impl FirestreamBackend for MockClient {
    fn list_templates(&self) -> BoxFuture<'_, ApiResult<Vec<Template>>> {
        Box::pin(async move {
            Ok(vec![
                // PySpark Template
                Template {
                    id: "pyspark-base".to_string(),
                    name: "PySpark Application".to_string(),
                    template_type: TemplateType::PySpark,
                    version: "1.0.0".to_string(),
                    description: Some("Full-featured PySpark application template with Kubernetes support".to_string()),
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
                        kafka: Some(KafkaConfig {
                            topics: vec!["input-topic".to_string(), "output-topic".to_string()],
                        }),
                        input_schema: None,
                        output_structure: None,
                    },
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                // Spark Scala Template
                Template {
                    id: "spark-scala-base".to_string(),
                    name: "Spark Scala Application".to_string(),
                    template_type: TemplateType::PySparkScala,
                    version: "1.0.0".to_string(),
                    description: Some("Full-featured Spark Scala application template with SBT build and Kubernetes support".to_string()),
                    config: FirestreamConfig {
                        app: AppConfig {
                            name: "spark-scala-app".to_string(),
                            version: "1.0.0".to_string(),
                            app_type: "spark-scala".to_string(),
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
            ])
        })
    }

    fn get_template(&self, id: &str) -> BoxFuture<'_, ApiResult<Template>> {
        let id = id.to_string();
        Box::pin(async move {
            let templates = vec![
                // PySpark Template
                Template {
                    id: "pyspark-base".to_string(),
                    name: "PySpark Application".to_string(),
                    template_type: TemplateType::PySpark,
                    version: "1.0.0".to_string(),
                    description: Some("Full-featured PySpark application template with Kubernetes support".to_string()),
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
                        kafka: Some(KafkaConfig {
                            topics: vec!["input-topic".to_string(), "output-topic".to_string()],
                        }),
                        input_schema: None,
                        output_structure: None,
                    },
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                // Spark Scala Template
                Template {
                    id: "spark-scala-base".to_string(),
                    name: "Spark Scala Application".to_string(),
                    template_type: TemplateType::PySparkScala,
                    version: "1.0.0".to_string(),
                    description: Some("Full-featured Spark Scala application template with SBT build and Kubernetes support".to_string()),
                    config: FirestreamConfig {
                        app: AppConfig {
                            name: "spark-scala-app".to_string(),
                            version: "1.0.0".to_string(),
                            app_type: "spark-scala".to_string(),
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
            ];
            
            templates.into_iter()
                .find(|t| t.id == id)
                .ok_or(ApiError::NotFound)
        })
    }

    fn list_deployments(&self) -> BoxFuture<'_, ApiResult<Vec<Deployment>>> {
        Box::pin(async move {
            Ok(vec![
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
                Deployment {
                    id: "dep-2".to_string(),
                    name: "api-service".to_string(),
                    template_id: "spark-scala-base".to_string(),
                    namespace: "default".to_string(),
                    status: DeploymentStatus::Running,
                    replicas: ReplicaStatus {
                        desired: 2,
                        ready: 2,
                        available: 2,
                    },
                    created_at: Utc::now() - chrono::Duration::days(1),
                    updated_at: Utc::now() - chrono::Duration::hours(1),
                },
                Deployment {
                    id: "dep-3".to_string(),
                    name: "ml-trainer".to_string(),
                    template_id: "pyspark-base".to_string(),
                    namespace: "default".to_string(),
                    status: DeploymentStatus::Pending,
                    replicas: ReplicaStatus {
                        desired: 1,
                        ready: 0,
                        available: 0,
                    },
                    created_at: Utc::now() - chrono::Duration::minutes(10),
                    updated_at: Utc::now() - chrono::Duration::minutes(2),
                },
            ])
        })
    }

    fn get_deployment(&self, id: &str) -> BoxFuture<'_, ApiResult<DeploymentDetail>> {
        let id = id.to_string();
        Box::pin(async move {
            let deployments = vec![
                Deployment {
                    id: "dep-1".to_string(),
                    name: "etl-pipeline".to_string(),
                    template_id: "1".to_string(),
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
                Deployment {
                    id: "dep-2".to_string(),
                    name: "api-service".to_string(),
                    template_id: "2".to_string(),
                    namespace: "default".to_string(),
                    status: DeploymentStatus::Running,
                    replicas: ReplicaStatus {
                        desired: 2,
                        ready: 2,
                        available: 2,
                    },
                    created_at: Utc::now() - chrono::Duration::days(1),
                    updated_at: Utc::now() - chrono::Duration::hours(1),
                },
                Deployment {
                    id: "dep-3".to_string(),
                    name: "ml-trainer".to_string(),
                    template_id: "1".to_string(),
                    namespace: "default".to_string(),
                    status: DeploymentStatus::Pending,
                    replicas: ReplicaStatus {
                        desired: 1,
                        ready: 0,
                        available: 0,
                    },
                    created_at: Utc::now() - chrono::Duration::minutes(10),
                    updated_at: Utc::now() - chrono::Duration::minutes(2),
                },
            ];

            let deployment = deployments.into_iter()
                .find(|d| d.id == id)
                .ok_or(ApiError::NotFound)?;

            Ok(DeploymentDetail {
                deployment: deployment.clone(),
                pods: vec![
                    Pod {
                        name: format!("{}-7d4b9c", deployment.name),
                        status: PodStatus::Running,
                        node: "worker-02".to_string(),
                        containers: vec![
                            Container {
                                name: "main".to_string(),
                                image: "gcr.io/project/app:v2.3.1".to_string(),
                                cpu: "0.8".to_string(),
                                memory: "1.5Gi".to_string(),
                            }
                        ],
                    },
                ],
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

    fn scale_deployment(&self, id: &str, replicas: u32) -> BoxFuture<'_, ApiResult<Deployment>> {
        let id = id.to_string();
        Box::pin(async move {
            let deployments = vec![
                Deployment {
                    id: "dep-1".to_string(),
                    name: "etl-pipeline".to_string(),
                    template_id: "1".to_string(),
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
            ];
            
            let deployment = deployments.into_iter()
                .find(|d| d.id == id)
                .ok_or(ApiError::NotFound)?;
            
            let mut deployment = deployment;
            deployment.replicas.desired = replicas;
            deployment.updated_at = Utc::now();
            Ok(deployment)
        })
    }

    fn delete_deployment(&self, _id: &str) -> BoxFuture<'_, ApiResult<()>> {
        Box::pin(async move {
            Ok(())
        })
    }

    fn get_deployment_logs(&self, _id: &str, _tail: Option<u32>) -> BoxFuture<'_, ApiResult<String>> {
        Box::pin(async move {
            Ok(format!(
                "[{}] Starting processing batch 15234\n\
                 [{}] Read 5000 messages from kafka\n\
                 [{}] Applied transformations\n\
                 [{}] Writing to delta table\n\
                 [{}] Batch complete (4.1s)",
                chrono::Local::now().format("%H:%M:%S"),
                chrono::Local::now().format("%H:%M:%S"),
                chrono::Local::now().format("%H:%M:%S"),
                chrono::Local::now().format("%H:%M:%S"),
                chrono::Local::now().format("%H:%M:%S"),
            ))
        })
    }

    fn get_cluster_status(&self) -> BoxFuture<'_, ApiResult<ClusterStatus>> {
        Box::pin(async move {
            let mut services = HashMap::new();
            services.insert("airflow".to_string(), ServiceStatus {
                status: HealthStatus::Healthy,
                replicas: "1/1".to_string(),
                endpoint: Some("http://airflow.local".to_string()),
            });
            services.insert("superset".to_string(), ServiceStatus {
                status: HealthStatus::Healthy,
                replicas: "1/1".to_string(),
                endpoint: Some("http://superset.local".to_string()),
            });
            services.insert("kafka".to_string(), ServiceStatus {
                status: HealthStatus::Healthy,
                replicas: "3/3".to_string(),
                endpoint: Some("kafka://kafka.local:9092".to_string()),
            });

            Ok(ClusterStatus {
                name: "local-k3d".to_string(),
                provider: ClusterProvider::K3d,
                status: HealthStatus::Healthy,
                nodes: NodeSummary {
                    total: 4,
                    ready: 4,
                    gpu: 2,
                },
                resources: ClusterResourceUtilization {
                    cpu_usage: 42.0,
                    memory_usage: 71.0,
                    gpu_usage: Some(87.0),
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
            Ok(vec![
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
                Node {
                    id: "node-2".to_string(),
                    name: "worker-02".to_string(),
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
                Node {
                    id: "node-gpu-1".to_string(),
                    name: "worker-gpu-01".to_string(),
                    provider: NodeProvider::Gcp,
                    instance_type: "g2-standard-16".to_string(),
                    status: NodeStatus::Ready,
                    resources: NodeResources {
                        cpu: 16,
                        memory: "64Gi".to_string(),
                        gpu: Some(GpuInfo {
                            count: 1,
                            gpu_type: "nvidia-l4".to_string(),
                        }),
                    },
                    labels: HashMap::new(),
                    spot: false,
                    created_at: Utc::now() - chrono::Duration::days(5),
                },
            ])
        })
    }

    fn get_node(&self, id: &str) -> BoxFuture<'_, ApiResult<NodeDetail>> {
        let id = id.to_string();
        Box::pin(async move {
            let nodes = vec![
                Node {
                    id: "node-gpu-1".to_string(),
                    name: "worker-gpu-01".to_string(),
                    provider: NodeProvider::Gcp,
                    instance_type: "g2-standard-16".to_string(),
                    status: NodeStatus::Ready,
                    resources: NodeResources {
                        cpu: 16,
                        memory: "64Gi".to_string(),
                        gpu: Some(GpuInfo {
                            count: 1,
                            gpu_type: "nvidia-l4".to_string(),
                        }),
                    },
                    labels: HashMap::new(),
                    spot: false,
                    created_at: Utc::now() - chrono::Duration::days(5),
                },
            ];
            
            let node = nodes.into_iter()
                .find(|n| n.id == id)
                .ok_or(ApiError::NotFound)?;

            Ok(NodeDetail {
                node,
                pods: vec![],
                utilization: node::ResourceUtilization {
                    cpu: 75.5,
                    memory: 48.2,
                    gpu: Some(87.0),
                },
            })
        })
    }

    fn provision_node(&self, _provider: &str, _instance_type: &str) -> BoxFuture<'_, ApiResult<String>> {
        Box::pin(async move {
            Ok(format!("node-{}", uuid::Uuid::new_v4()))
        })
    }

    fn list_delta_tables(&self) -> BoxFuture<'_, ApiResult<Vec<DeltaTable>>> {
        Box::pin(async move {
            Ok(vec![
                DeltaTable {
                    name: "raw_events".to_string(),
                    database: "analytics".to_string(),
                    location: "s3://data/analytics/raw".to_string(),
                    format: "delta".to_string(),
                    partition_columns: vec!["date".to_string(), "hour".to_string()],
                    num_files: 234,
                    size_in_bytes: 1_800_000_000,
                    last_modified: Utc::now() - chrono::Duration::minutes(2),
                },
                DeltaTable {
                    name: "processed_events".to_string(),
                    database: "analytics".to_string(),
                    location: "s3://data/analytics/processed".to_string(),
                    format: "delta".to_string(),
                    partition_columns: vec!["date".to_string()],
                    num_files: 156,
                    size_in_bytes: 1_200_000_000,
                    last_modified: Utc::now() - chrono::Duration::minutes(5),
                },
            ])
        })
    }

    fn get_table_schema(&self, table_name: &str) -> BoxFuture<'_, ApiResult<TableSchema>> {
        let table_name = table_name.to_string();
        Box::pin(async move {
            Ok(TableSchema {
                table_name,
                columns: vec![
                    Column {
                        name: "event_id".to_string(),
                        column_type: "string".to_string(),
                        nullable: false,
                        metadata: None,
                    },
                    Column {
                        name: "timestamp".to_string(),
                        column_type: "datetime".to_string(),
                        nullable: false,
                        metadata: None,
                    },
                    Column {
                        name: "user_id".to_string(),
                        column_type: "string".to_string(),
                        nullable: true,
                        metadata: None,
                    },
                ],
            })
        })
    }

    fn list_lakefs_branches(&self) -> BoxFuture<'_, ApiResult<Vec<LakeFSBranch>>> {
        Box::pin(async move {
            Ok(vec![
                LakeFSBranch {
                    name: "main".to_string(),
                    repository: "analytics".to_string(),
                    commit_id: "abc123".to_string(),
                    created_at: Utc::now() - chrono::Duration::days(30),
                    creator: "system".to_string(),
                },
                LakeFSBranch {
                    name: "feature-123".to_string(),
                    repository: "analytics".to_string(),
                    commit_id: "def456".to_string(),
                    created_at: Utc::now() - chrono::Duration::days(2),
                    creator: "user@example.com".to_string(),
                },
            ])
        })
    }

    fn list_s3_buckets(&self) -> BoxFuture<'_, ApiResult<Vec<S3Bucket>>> {
        Box::pin(async move {
            Ok(vec![
                S3Bucket {
                    name: "data-lake".to_string(),
                    creation_date: Utc::now() - chrono::Duration::days(180),
                    size_in_bytes: 5_000_000_000,
                    object_count: 15234,
                },
                S3Bucket {
                    name: "models".to_string(),
                    creation_date: Utc::now() - chrono::Duration::days(90),
                    size_in_bytes: 2_000_000_000,
                    object_count: 543,
                },
            ])
        })
    }

    fn start_build(&self, _deployment_id: &str) -> BoxFuture<'_, ApiResult<BuildStatus>> {
        Box::pin(async move {
            Ok(BuildStatus {
                build_id: format!("build-{}", uuid::Uuid::new_v4()),
                status: BuildState::Building,
                progress: 25.0,
                current_step: Some("Installing dependencies".to_string()),
                total_steps: Some(12),
                started_at: Utc::now(),
                completed_at: None,
                image: None,
                digest: None,
            })
        })
    }

    fn get_build_status(&self, build_id: &str) -> BoxFuture<'_, ApiResult<BuildStatus>> {
        let build_id = build_id.to_string();
        Box::pin(async move {
            Ok(BuildStatus {
                build_id,
                status: BuildState::Building,
                progress: 78.0,
                current_step: Some("Installing dependencies".to_string()),
                total_steps: Some(12),
                started_at: Utc::now() - chrono::Duration::minutes(2),
                completed_at: None,
                image: None,
                digest: None,
            })
        })
    }

    fn get_build_logs(&self, _build_id: &str) -> BoxFuture<'_, ApiResult<String>> {
        Box::pin(async move {
            Ok(
                "✓ Generated Dockerfile\n\
                 ✓ Copied source files\n\
                 ✓ Building base image\n\
                 ▶ Installing dependencies\n\
                 \n\
                 Step 4/12: RUN pip install -r requirements\n\
                 Collecting pyspark==3.5.0\n\
                 Downloading pyspark-3.5.0.tar.gz (316 MB)\n\
                 ████████████████████░░░░░ 78% 247MB/316MB\n\
                 \n\
                 Collecting kafka-python==2.0.2\n\
                 Downloading kafka-python-2.0.2.tar.gz\n\
                 Installing collected packages...".to_string()
            )
        })
    }

    fn list_secrets(&self) -> BoxFuture<'_, ApiResult<Vec<SecretInfo>>> {
        Box::pin(async move {
            Ok(vec![
                SecretInfo {
                    name: "api-keys".to_string(),
                    namespace: "default".to_string(),
                    secret_type: SecretType::Opaque,
                    keys: vec!["api_key".to_string(), "api_secret".to_string()],
                    created_at: Utc::now() - chrono::Duration::days(10),
                    updated_at: Utc::now() - chrono::Duration::days(2),
                },
                SecretInfo {
                    name: "tls-cert".to_string(),
                    namespace: "default".to_string(),
                    secret_type: SecretType::TLS,
                    keys: vec!["tls.crt".to_string(), "tls.key".to_string()],
                    created_at: Utc::now() - chrono::Duration::days(30),
                    updated_at: Utc::now() - chrono::Duration::days(30),
                },
                SecretInfo {
                    name: "db-credentials".to_string(),
                    namespace: "default".to_string(),
                    secret_type: SecretType::Opaque,
                    keys: vec!["username".to_string(), "password".to_string(), "host".to_string(), "port".to_string()],
                    created_at: Utc::now() - chrono::Duration::days(5),
                    updated_at: Utc::now() - chrono::Duration::days(1),
                },
            ])
        })
    }
    
    fn get_secret(&self, id: &str) -> BoxFuture<'_, ApiResult<SecretInfo>> {
        let id = id.to_string();
        Box::pin(async move {
            let secrets = vec![
                SecretInfo {
                    name: "api-keys".to_string(),
                    namespace: "default".to_string(),
                    secret_type: SecretType::Opaque,
                    keys: vec!["api_key".to_string(), "api_secret".to_string()],
                    created_at: Utc::now() - chrono::Duration::days(10),
                    updated_at: Utc::now() - chrono::Duration::days(2),
                },
                SecretInfo {
                    name: "tls-cert".to_string(),
                    namespace: "default".to_string(),
                    secret_type: SecretType::TLS,
                    keys: vec!["tls.crt".to_string(), "tls.key".to_string()],
                    created_at: Utc::now() - chrono::Duration::days(30),
                    updated_at: Utc::now() - chrono::Duration::days(30),
                },
                SecretInfo {
                    name: "db-credentials".to_string(),
                    namespace: "default".to_string(),
                    secret_type: SecretType::Opaque,
                    keys: vec!["username".to_string(), "password".to_string(), "host".to_string(), "port".to_string()],
                    created_at: Utc::now() - chrono::Duration::days(5),
                    updated_at: Utc::now() - chrono::Duration::days(1),
                },
            ];
            
            secrets.into_iter()
                .find(|s| s.name == id)
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
    
    // Iceberg operations
    
    fn list_iceberg_catalogs(&self) -> BoxFuture<'_, ApiResult<Vec<IcebergCatalog>>> {
        Box::pin(async move {
            Ok(vec![
                IcebergCatalog {
                    name: "local".to_string(),
                    catalog_type: IcebergCatalogType::Memory,
                    warehouse: "/tmp/iceberg-warehouse".to_string(),
                    namespaces: vec!["default".to_string(), "analytics".to_string()],
                },
                IcebergCatalog {
                    name: "s3_prod".to_string(),
                    catalog_type: IcebergCatalogType::Rest,
                    warehouse: "s3://data-lake/iceberg".to_string(),
                    namespaces: vec!["prod".to_string(), "staging".to_string()],
                },
            ])
        })
    }
    
    fn get_iceberg_catalog(&self, name: &str) -> BoxFuture<'_, ApiResult<IcebergCatalog>> {
        let name = name.to_string();
        Box::pin(async move {
            match name.as_str() {
                "local" => Ok(IcebergCatalog {
                    name: "local".to_string(),
                    catalog_type: IcebergCatalogType::Memory,
                    warehouse: "/tmp/iceberg-warehouse".to_string(),
                    namespaces: vec!["default".to_string(), "analytics".to_string()],
                }),
                "s3_prod" => Ok(IcebergCatalog {
                    name: "s3_prod".to_string(),
                    catalog_type: IcebergCatalogType::Rest,
                    warehouse: "s3://data-lake/iceberg".to_string(),
                    namespaces: vec!["prod".to_string(), "staging".to_string()],
                }),
                _ => Err(ApiError::NotFound),
            }
        })
    }
    
    fn create_iceberg_catalog(&self, config: &StorageConfig) -> BoxFuture<'_, ApiResult<IcebergCatalog>> {
        let config = config.clone();
        Box::pin(async move {
            let catalog_name = match &config.storage_type {
                StorageType::LocalFileSystem => "new_local",
                StorageType::S3 => "new_s3",
                StorageType::GoogleCloudStorage => "new_gcs",
            };
            
            Ok(IcebergCatalog {
                name: catalog_name.to_string(),
                catalog_type: IcebergCatalogType::Memory,
                warehouse: "/tmp/new-catalog".to_string(),
                namespaces: vec![],
            })
        })
    }
    
    fn list_iceberg_namespaces(&self, catalog: &str) -> BoxFuture<'_, ApiResult<Vec<IcebergNamespace>>> {
        let catalog = catalog.to_string();
        Box::pin(async move {
            match catalog.as_str() {
                "local" => Ok(vec![
                    IcebergNamespace {
                        name: "default".to_string(),
                        catalog: "local".to_string(),
                        tables: vec!["users".to_string(), "events".to_string()],
                        properties: HashMap::new(),
                    },
                    IcebergNamespace {
                        name: "analytics".to_string(),
                        catalog: "local".to_string(),
                        tables: vec!["pageviews".to_string(), "sessions".to_string()],
                        properties: HashMap::new(),
                    },
                ]),
                "s3_prod" => Ok(vec![
                    IcebergNamespace {
                        name: "prod".to_string(),
                        catalog: "s3_prod".to_string(),
                        tables: vec!["transactions".to_string(), "customers".to_string()],
                        properties: HashMap::new(),
                    },
                ]),
                _ => Ok(vec![]),
            }
        })
    }
    
    fn create_iceberg_namespace(&self, catalog: &str, namespace: &str, properties: HashMap<String, String>) -> BoxFuture<'_, ApiResult<IcebergNamespace>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        Box::pin(async move {
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
            match (catalog.as_str(), namespace.as_str()) {
                ("local", "default") => Ok(vec![
                    IcebergTable {
                        id: "local.default.users".to_string(),
                        name: "users".to_string(),
                        namespace: "default".to_string(),
                        catalog: "local".to_string(),
                        location: "/tmp/iceberg-warehouse/default/users".to_string(),
                        current_snapshot_id: Some(12345),
                        schema: IcebergSchema {
                            schema_id: 1,
                            fields: vec![
                                IcebergField {
                                    id: 1,
                                    name: "id".to_string(),
                                    field_type: "long".to_string(),
                                    required: true,
                                    doc: None,
                                },
                                IcebergField {
                                    id: 2,
                                    name: "username".to_string(),
                                    field_type: "string".to_string(),
                                    required: true,
                                    doc: None,
                                },
                                IcebergField {
                                    id: 3,
                                    name: "email".to_string(),
                                    field_type: "string".to_string(),
                                    required: false,
                                    doc: None,
                                },
                                IcebergField {
                                    id: 4,
                                    name: "created_at".to_string(),
                                    field_type: "timestamp".to_string(),
                                    required: true,
                                    doc: None,
                                },
                            ],
                        },
                        partition_spec: None,
                        properties: HashMap::new(),
                        last_modified: Utc::now() - chrono::Duration::hours(2),
                    },
                    IcebergTable {
                        id: "local.default.events".to_string(),
                        name: "events".to_string(),
                        namespace: "default".to_string(),
                        catalog: "local".to_string(),
                        location: "/tmp/iceberg-warehouse/default/events".to_string(),
                        current_snapshot_id: Some(54321),
                        schema: IcebergSchema {
                            schema_id: 1,
                            fields: vec![
                                IcebergField {
                                    id: 1,
                                    name: "event_id".to_string(),
                                    field_type: "uuid".to_string(),
                                    required: true,
                                    doc: None,
                                },
                                IcebergField {
                                    id: 2,
                                    name: "user_id".to_string(),
                                    field_type: "long".to_string(),
                                    required: true,
                                    doc: None,
                                },
                                IcebergField {
                                    id: 3,
                                    name: "event_type".to_string(),
                                    field_type: "string".to_string(),
                                    required: true,
                                    doc: None,
                                },
                                IcebergField {
                                    id: 4,
                                    name: "timestamp".to_string(),
                                    field_type: "timestamptz".to_string(),
                                    required: true,
                                    doc: None,
                                },
                            ],
                        },
                        partition_spec: Some(vec![
                            PartitionField {
                                source_id: 4,
                                field_id: 1000,
                                name: "timestamp_day".to_string(),
                                transform: "day".to_string(),
                            },
                        ]),
                        properties: HashMap::new(),
                        last_modified: Utc::now() - chrono::Duration::minutes(30),
                    },
                ]),
                _ => Ok(vec![]),
            }
        })
    }
    
    fn get_iceberg_table(&self, catalog: &str, namespace: &str, table: &str) -> BoxFuture<'_, ApiResult<IcebergTable>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        let table = table.to_string();
        Box::pin(async move {
            if catalog == "local" && namespace == "default" && table == "users" {
                Ok(IcebergTable {
                    id: "local.default.users".to_string(),
                    name: "users".to_string(),
                    namespace: "default".to_string(),
                    catalog: "local".to_string(),
                    location: "/tmp/iceberg-warehouse/default/users".to_string(),
                    current_snapshot_id: Some(12345),
                    schema: IcebergSchema {
                        schema_id: 1,
                        fields: vec![
                            IcebergField {
                                id: 1,
                                name: "id".to_string(),
                                field_type: "long".to_string(),
                                required: true,
                                doc: None,
                            },
                            IcebergField {
                                id: 2,
                                name: "username".to_string(),
                                field_type: "string".to_string(),
                                required: true,
                                doc: None,
                            },
                            IcebergField {
                                id: 3,
                                name: "email".to_string(),
                                field_type: "string".to_string(),
                                required: false,
                                doc: None,
                            },
                            IcebergField {
                                id: 4,
                                name: "created_at".to_string(),
                                field_type: "timestamp".to_string(),
                                required: true,
                                doc: None,
                            },
                        ],
                    },
                    partition_spec: None,
                    properties: HashMap::new(),
                    last_modified: Utc::now() - chrono::Duration::hours(2),
                })
            } else {
                Err(ApiError::NotFound)
            }
        })
    }
    
    fn create_iceberg_table(&self, catalog: &str, namespace: &str, table: &str, schema: IcebergSchema, partition_spec: Option<Vec<PartitionField>>) -> BoxFuture<'_, ApiResult<IcebergTable>> {
        let catalog = catalog.to_string();
        let namespace = namespace.to_string();
        let table = table.to_string();
        Box::pin(async move {
            Ok(IcebergTable {
                id: format!("{}.{}.{}", catalog, namespace, table),
                name: table.clone(),
                namespace: namespace.clone(),
                catalog: catalog.clone(),
                location: format!("/tmp/iceberg-warehouse/{}/{}", namespace, table),
                current_snapshot_id: None,
                schema,
                partition_spec,
                properties: HashMap::new(),
                last_modified: Utc::now(),
            })
        })
    }
    
    fn drop_iceberg_table(&self, _catalog: &str, _namespace: &str, _table: &str) -> BoxFuture<'_, ApiResult<()>> {
        Box::pin(async move {
            Ok(())
        })
    }
    
    fn query_iceberg_table(&self, _catalog: &str, _namespace: &str, table: &str, sql: &str) -> BoxFuture<'_, ApiResult<IcebergQueryResult>> {
        let table = table.to_string();
        let _sql = sql.to_string();
        Box::pin(async move {
            // Mock query results
            if table == "users" {
                Ok(IcebergQueryResult {
                    columns: vec!["id".to_string(), "username".to_string(), "email".to_string(), "created_at".to_string()],
                    rows: vec![
                        vec![
                            serde_json::json!(1),
                            serde_json::json!("alice"),
                            serde_json::json!("alice@example.com"),
                            serde_json::json!("2024-01-15T10:30:00Z"),
                        ],
                        vec![
                            serde_json::json!(2),
                            serde_json::json!("bob"),
                            serde_json::json!("bob@example.com"),
                            serde_json::json!("2024-01-16T14:20:00Z"),
                        ],
                    ],
                    row_count: 2,
                })
            } else {
                Ok(IcebergQueryResult {
                    columns: vec![],
                    rows: vec![],
                    row_count: 0,
                })
            }
        })
    }
    
    fn preview_iceberg_table(&self, catalog: &str, namespace: &str, table: &str, limit: usize) -> BoxFuture<'_, ApiResult<IcebergQueryResult>> {
        let _catalog = catalog.to_string();
        let _namespace = namespace.to_string();
        let table = table.to_string();
        Box::pin(async move {
            // Mock preview results
            if table == "users" {
                Ok(IcebergQueryResult {
                    columns: vec!["id".to_string(), "username".to_string(), "email".to_string(), "created_at".to_string()],
                    rows: vec![
                        vec![
                            serde_json::json!(1),
                            serde_json::json!("alice"),
                            serde_json::json!("alice@example.com"),
                            serde_json::json!("2024-01-15T10:30:00Z"),
                        ],
                        vec![
                            serde_json::json!(2),
                            serde_json::json!("bob"),
                            serde_json::json!("bob@example.com"),
                            serde_json::json!("2024-01-16T14:20:00Z"),
                        ],
                    ].into_iter().take(limit).collect(),
                    row_count: 2.min(limit),
                })
            } else {
                Ok(IcebergQueryResult {
                    columns: vec![],
                    rows: vec![],
                    row_count: 0,
                })
            }
        })
    }
}

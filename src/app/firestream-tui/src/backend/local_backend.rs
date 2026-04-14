//! Local Docker backend implementing FirestreamBackend.
//!
//! Uses docker-manager for container lifecycle and nix-container-builder for image builds.
//! All operations go through the local Docker socket.

use super::bootstrap::BootstrapState;
use super::build_queue::BuildQueue;
use super::manifest::{ContainerManifest, ManifestRegistry};
use super::{ApiError, ApiResult, BoxFuture, FirestreamBackend};
use crate::event::{AppEvent, Event};
use crate::models::*;
use docker_manager::DockerManager;
use nix_container_builder::NixContainerBuilder;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Backend that talks directly to the local Docker daemon.
pub struct LocalBackend {
    docker: DockerManager,
    #[allow(dead_code)]
    builder: NixContainerBuilder,
    repo_root: PathBuf,
    manifests: Vec<ContainerManifest>,
    build_queue: Arc<Mutex<BuildQueue>>,
    event_sender: mpsc::UnboundedSender<Event>,
    bootstrap: BootstrapState,
}

impl LocalBackend {
    /// Create a new LocalBackend.
    pub async fn new(
        docker: DockerManager,
        builder: NixContainerBuilder,
        repo_root: PathBuf,
        bootstrap: BootstrapState,
        event_sender: mpsc::UnboundedSender<Event>,
    ) -> Self {
        let registry = ManifestRegistry::discover(&repo_root);
        let manifests = registry.all().to_vec();

        Self {
            docker,
            builder,
            repo_root,
            manifests,
            build_queue: Arc::new(Mutex::new(BuildQueue::new())),
            event_sender,
            bootstrap,
        }
    }

    /// Get the bootstrap state.
    pub fn bootstrap(&self) -> &BootstrapState {
        &self.bootstrap
    }

    /// Helper: create a Template from a ContainerManifest.
    fn manifest_to_template(m: &ContainerManifest, built: bool) -> Template {
        Template {
            id: m.name.clone(),
            name: m.name.clone(),
            template_type: TemplateType::Python, // Reuse existing enum
            version: m.image_tag.clone(),
            description: Some(format!(
                "{}:{} [{}]",
                m.image_name,
                m.image_tag,
                if built { "built" } else { "not built" },
            )),
            config: FirestreamConfig {
                app: AppConfig {
                    name: m.name.clone(),
                    version: m.image_tag.clone(),
                    app_type: "container".to_string(),
                },
                resources: ResourceConfig {
                    cpu: "1".to_string(),
                    memory: "1Gi".to_string(),
                    gpu: None,
                    gpu_type: None,
                },
                kafka: None,
                input_schema: None,
                output_structure: None,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Helper: create a Deployment from a Docker container.
    fn docker_container_to_deployment(
        name: &str,
        id: &str,
        running: bool,
    ) -> Deployment {
        Deployment {
            id: id.to_string(),
            name: name.to_string(),
            template_id: String::new(),
            namespace: "local".to_string(),
            status: if running { DeploymentStatus::Running } else { DeploymentStatus::Pending },
            replicas: ReplicaStatus {
                desired: 1,
                ready: if running { 1 } else { 0 },
                available: if running { 1 } else { 0 },
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl FirestreamBackend for LocalBackend {
    // ── Templates (container manifests) ─────────────────────────────
    fn list_templates(&self) -> BoxFuture<'_, ApiResult<Vec<Template>>> {
        Box::pin(async move {
            Ok(self.manifests.iter().map(|m| {
                let built = self.bootstrap.built_images.contains_key(&m.name);
                Self::manifest_to_template(m, built)
            }).collect())
        })
    }

    fn get_template(&self, id: &str) -> BoxFuture<'_, ApiResult<Template>> {
        let id = id.to_string();
        Box::pin(async move {
            let m = self.manifests.iter().find(|m| m.name == id).ok_or(ApiError::NotFound)?;
            let built = self.bootstrap.built_images.contains_key(&id);
            Ok(Self::manifest_to_template(m, built))
        })
    }

    // ── Deployments (running compose services) ──────────────────────
    fn list_deployments(&self) -> BoxFuture<'_, ApiResult<Vec<Deployment>>> {
        Box::pin(async move {
            let mut deployments = Vec::new();
            if let Ok(containers) = self.docker.list_containers(false).await {
                for c in containers {
                    let names = c.names.as_ref().map(|n| n.join(", ")).unwrap_or_default();
                    if names.contains("firestream") {
                        let running = c.state.as_deref() == Some("running");
                        let id = c.id.clone().unwrap_or_default();
                        let name = names.trim_start_matches('/').to_string();
                        deployments.push(Self::docker_container_to_deployment(&name, &id, running));
                    }
                }
            }
            Ok(deployments)
        })
    }

    fn get_deployment(&self, id: &str) -> BoxFuture<'_, ApiResult<DeploymentDetail>> {
        let id = id.to_string();
        Box::pin(async move {
            Ok(DeploymentDetail {
                deployment: Self::docker_container_to_deployment(&id, &id, true),
                pods: vec![],
                services: vec![],
                ingress: vec![],
            })
        })
    }

    fn create_deployment(&self, template_id: &str, _name: &str, _payload: serde_json::Value) -> BoxFuture<'_, ApiResult<Deployment>> {
        let template_id = template_id.to_string();
        Box::pin(async move {
            let manifest = self.manifests.iter()
                .find(|m| m.name == template_id)
                .ok_or(ApiError::NotFound)?;

            if let Some(ref wd) = manifest.compose_working_dir {
                let full = self.repo_root.join(wd);
                self.docker.compose_up(&full, true, None).await
                    .map_err(|e| ApiError::Server { message: e.to_string() })?;
            }

            Ok(Self::docker_container_to_deployment(&template_id, &template_id, true))
        })
    }

    fn scale_deployment(&self, _id: &str, _replicas: u32) -> BoxFuture<'_, ApiResult<Deployment>> {
        Box::pin(async move {
            Err(ApiError::BadRequest("Scaling not supported in local Docker mode".to_string()))
        })
    }

    fn delete_deployment(&self, id: &str) -> BoxFuture<'_, ApiResult<()>> {
        let id = id.to_string();
        Box::pin(async move {
            let manifest = self.manifests.iter()
                .find(|m| m.name == id)
                .ok_or(ApiError::NotFound)?;

            if let Some(ref wd) = manifest.compose_working_dir {
                let full = self.repo_root.join(wd);
                self.docker.compose_down(&full, false, true).await
                    .map_err(|e| ApiError::Server { message: e.to_string() })?;
            }
            Ok(())
        })
    }

    fn get_deployment_logs(&self, id: &str, tail: Option<u32>) -> BoxFuture<'_, ApiResult<String>> {
        let id = id.to_string();
        Box::pin(async move {
            let manifest = self.manifests.iter()
                .find(|m| m.name == id)
                .ok_or(ApiError::NotFound)?;

            if let Some(ref wd) = manifest.compose_working_dir {
                let full = self.repo_root.join(wd);
                let tail_str = tail.map(|t| t.to_string());
                let logs = self.docker.compose_logs(&full, false, tail_str.as_deref(), None).await
                    .map_err(|e| ApiError::Server { message: e.to_string() })?;
                Ok(logs)
            } else {
                Ok(String::new())
            }
        })
    }

    // ── Cluster (Docker system info) ────────────────────────────────
    fn get_cluster_status(&self) -> BoxFuture<'_, ApiResult<ClusterStatus>> {
        Box::pin(async move {
            let docker_ver = self.bootstrap.docker_version.clone().unwrap_or_else(|| "unknown".to_string());
            Ok(ClusterStatus {
                name: format!("local-docker ({})", docker_ver),
                provider: ClusterProvider::K3d, // Closest match for local
                status: if self.bootstrap.docker_healthy { HealthStatus::Healthy } else { HealthStatus::Degraded },
                nodes: NodeSummary { total: 1, ready: 1, gpu: 0 },
                resources: ClusterResourceUtilization {
                    cpu_usage: 0.0,
                    memory_usage: 0.0,
                    gpu_usage: None,
                },
                services: HashMap::new(),
            })
        })
    }

    fn list_services(&self) -> BoxFuture<'_, ApiResult<Vec<Service>>> {
        Box::pin(async move {
            let mut services = Vec::new();
            if let Ok(containers) = self.docker.list_containers(false).await {
                for c in containers {
                    let names = c.names.as_ref().map(|n| n.join(", ")).unwrap_or_default();
                    if names.contains("firestream") {
                        services.push(Service {
                            name: names.trim_start_matches('/').to_string(),
                            service_type: ServiceType::ClusterIP,
                            ports: vec![],
                            selector: HashMap::new(),
                            endpoints: vec!["localhost".to_string()],
                        });
                    }
                }
            }
            Ok(services)
        })
    }

    // ── Nodes (single local node) ───────────────────────────────────
    fn list_nodes(&self) -> BoxFuture<'_, ApiResult<Vec<Node>>> {
        Box::pin(async move {
            Ok(vec![Node {
                id: "local".to_string(),
                name: "local-docker".to_string(),
                provider: NodeProvider::Local,
                instance_type: self.bootstrap.platform_arch.clone(),
                status: NodeStatus::Ready,
                resources: NodeResources {
                    cpu: 4,
                    memory: "16Gi".to_string(),
                    gpu: None,
                },
                labels: HashMap::new(),
                spot: false,
                created_at: chrono::Utc::now(),
            }])
        })
    }

    fn get_node(&self, _id: &str) -> BoxFuture<'_, ApiResult<NodeDetail>> {
        Box::pin(async move {
            Ok(NodeDetail {
                node: Node {
                    id: "local".to_string(),
                    name: "local-docker".to_string(),
                    provider: NodeProvider::Local,
                    instance_type: self.bootstrap.platform_arch.clone(),
                    status: NodeStatus::Ready,
                    resources: NodeResources { cpu: 4, memory: "16Gi".to_string(), gpu: None },
                    labels: HashMap::new(),
                    spot: false,
                    created_at: chrono::Utc::now(),
                },
                pods: vec![],
                utilization: ResourceUtilization { cpu: 0.0, memory: 0.0, gpu: None },
            })
        })
    }

    fn provision_node(&self, _provider: &str, _instance_type: &str) -> BoxFuture<'_, ApiResult<String>> {
        Box::pin(async move {
            Err(ApiError::BadRequest("Not available in local Docker mode".to_string()))
        })
    }

    // ── Data (not applicable locally) ───────────────────────────────
    fn list_delta_tables(&self) -> BoxFuture<'_, ApiResult<Vec<DeltaTable>>> {
        Box::pin(async move { Ok(vec![]) })
    }
    fn get_table_schema(&self, _: &str) -> BoxFuture<'_, ApiResult<TableSchema>> {
        Box::pin(async move { Err(ApiError::NotFound) })
    }
    fn list_lakefs_branches(&self) -> BoxFuture<'_, ApiResult<Vec<LakeFSBranch>>> {
        Box::pin(async move { Ok(vec![]) })
    }
    fn list_s3_buckets(&self) -> BoxFuture<'_, ApiResult<Vec<S3Bucket>>> {
        Box::pin(async move { Ok(vec![]) })
    }

    // ── Builds ──────────────────────────────────────────────────────
    fn start_build(&self, deployment_id: &str) -> BoxFuture<'_, ApiResult<BuildStatus>> {
        let name = deployment_id.to_string();
        Box::pin(async move {
            let manifest = self.manifests.iter()
                .find(|m| m.name == name)
                .ok_or(ApiError::NotFound)?;

            let build_id = format!("build-{}", uuid::Uuid::new_v4());
            let pkg = manifest.nix_package.clone();

            {
                let mut queue = self.build_queue.lock().await;
                let pos = queue.enqueue(name.clone(), pkg);
                let _ = self.event_sender.send(Event::App(AppEvent::BuildQueued {
                    container: name.clone(),
                    position: pos,
                }));
            }

            Ok(BuildStatus {
                build_id,
                status: BuildState::Pending,
                progress: 0.0,
                current_step: Some("Queued".to_string()),
                total_steps: None,
                started_at: chrono::Utc::now(),
                completed_at: None,
                image: None,
                digest: None,
            })
        })
    }

    fn get_build_status(&self, build_id: &str) -> BoxFuture<'_, ApiResult<BuildStatus>> {
        let bid = build_id.to_string();
        Box::pin(async move {
            let queue = self.build_queue.lock().await;
            if let Some(active) = queue.active() {
                Ok(BuildStatus {
                    build_id: bid,
                    status: BuildState::Building,
                    progress: 50.0,
                    current_step: Some(format!("{:?}", active.phase)),
                    total_steps: None,
                    started_at: chrono::Utc::now(),
                    completed_at: None,
                    image: None,
                    digest: None,
                })
            } else {
                Err(ApiError::NotFound)
            }
        })
    }

    fn get_build_logs(&self, _build_id: &str) -> BoxFuture<'_, ApiResult<String>> {
        Box::pin(async move {
            let queue = self.build_queue.lock().await;
            if let Some(active) = queue.active() {
                Ok(active.log_lines.join("\n"))
            } else {
                Ok(String::new())
            }
        })
    }

    // ── Secrets (not applicable locally) ────────────────────────────
    fn list_secrets(&self) -> BoxFuture<'_, ApiResult<Vec<SecretInfo>>> {
        Box::pin(async move { Ok(vec![]) })
    }
    fn get_secret(&self, _: &str) -> BoxFuture<'_, ApiResult<SecretInfo>> {
        Box::pin(async move { Err(ApiError::NotFound) })
    }
    fn create_secret(&self, _: &str, _: HashMap<String, String>) -> BoxFuture<'_, ApiResult<SecretInfo>> {
        Box::pin(async move { Err(ApiError::BadRequest("Not available locally".to_string())) })
    }
    fn delete_secret(&self, _: &str) -> BoxFuture<'_, ApiResult<()>> {
        Box::pin(async move { Err(ApiError::BadRequest("Not available locally".to_string())) })
    }
}

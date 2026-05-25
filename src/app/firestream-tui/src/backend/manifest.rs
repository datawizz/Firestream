//! Container manifest discovery and registry.
//!
//! Discovers container metadata from the filesystem (docker-compose.yml, module.nix, etc.)
//! and provides a registry for dependency resolution and build ordering.

use nix_container_builder::BuildConfig;
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Credentials for a container service.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub url: String,
    pub username: String,
    pub password: String,
}

/// Complete metadata for a container that can be built and run.
#[derive(Debug, Clone)]
pub struct ContainerManifest {
    /// Container name (e.g., "airflow")
    pub name: String,
    /// Root flake package name (e.g., "airflow")
    pub nix_package: String,
    /// Path to docker-compose.yml (if exists)
    pub compose_path: Option<PathBuf>,
    /// Working directory for compose commands (directory containing compose file)
    pub compose_working_dir: Option<PathBuf>,
    /// Docker image name (e.g., "firestream-airflow")
    pub image_name: String,
    /// Docker image tag (e.g., "3.0.3")
    pub image_tag: String,
    /// Container dependencies (names of containers that must be running)
    pub dependencies: Vec<String>,
    /// Service credentials
    pub credentials: Option<Credentials>,
    /// Port mappings (host, container)
    pub ports: Vec<(u16, u16)>,
    /// Environment variable overrides for compose (e.g., PG_VERSION=17)
    pub env_overrides: HashMap<String, String>,
}

/// Registry of all known container manifests.
pub struct ManifestRegistry {
    manifests: Vec<ContainerManifest>,
}

impl ManifestRegistry {
    /// Discover container manifests from the repository filesystem.
    pub fn discover(repo_root: &Path) -> Self {
        let containers_dir = repo_root.join("src/containers/firestream");
        let config = BuildConfig::default();
        let mut manifests = Vec::new();

        if !containers_dir.exists() {
            tracing::warn!("Containers directory not found, using defaults");
            return Self { manifests: default_manifests(repo_root) };
        }

        let entries = match std::fs::read_dir(&containers_dir) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::warn!("Failed to read containers directory: {}", e);
                return Self { manifests: default_manifests(repo_root) };
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();

            // Try to build a manifest from filesystem inspection
            if let Some(manifest) = build_manifest_from_dir(&name, &path, &config, repo_root) {
                manifests.push(manifest);
            }
        }

        manifests.sort_by(|a, b| a.name.cmp(&b.name));

        if manifests.is_empty() {
            tracing::warn!("No containers discovered, using defaults");
            return Self { manifests: default_manifests(repo_root) };
        }

        Self { manifests }
    }

    /// Get all manifests.
    pub fn all(&self) -> &[ContainerManifest] {
        &self.manifests
    }

    /// Find a manifest by container name.
    pub fn get(&self, name: &str) -> Option<&ContainerManifest> {
        self.manifests.iter().find(|m| m.name == name)
    }

    /// Compute a topological build order for a container and its dependencies.
    /// Returns container names in the order they should be built.
    pub fn build_order(&self, target: &str) -> Vec<String> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.topo_visit(target, &mut order, &mut visited);
        order
    }

    fn topo_visit(
        &self,
        name: &str,
        order: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(name) {
            return;
        }
        visited.insert(name.to_string());

        if let Some(manifest) = self.get(name) {
            for dep in &manifest.dependencies {
                self.topo_visit(dep, order, visited);
            }
        }
        order.push(name.to_string());
    }
}

/// Try to build a ContainerManifest from a container directory.
fn build_manifest_from_dir(
    name: &str,
    path: &Path,
    config: &BuildConfig,
    repo_root: &Path,
) -> Option<ContainerManifest> {
    // Find the compose file (might be at root or in a versioned subdir)
    let compose_path = find_compose_file(path);
    let compose_working_dir = compose_path.as_ref().and_then(|p| p.parent().map(|d| d.to_path_buf()));

    // Resolve the nix package name from registry
    let nix_package = config.resolve_package_name(name, None)?;

    // Parse compose file for image tag, ports, dependencies
    let (image_name, image_tag, ports, dependencies, credentials, env_overrides) =
        if let Some(ref cp) = compose_path {
            parse_compose_file(cp, name)
        } else {
            (
                format!("firestream-{}", name),
                "latest".to_string(),
                vec![],
                vec![],
                None,
                HashMap::new(),
            )
        };

    Some(ContainerManifest {
        name: name.to_string(),
        nix_package,
        compose_path: compose_path.map(|p| {
            p.strip_prefix(repo_root).map(|r| r.to_path_buf()).unwrap_or(p)
        }),
        compose_working_dir: compose_working_dir.map(|p| {
            p.strip_prefix(repo_root).map(|r| r.to_path_buf()).unwrap_or(p)
        }),
        image_name,
        image_tag,
        dependencies,
        credentials,
        ports,
        env_overrides,
    })
}

/// Find docker-compose.yml in a container directory (root or versioned subdir).
fn find_compose_file(container_dir: &Path) -> Option<PathBuf> {
    // Check root level first
    let root_compose = container_dir.join("docker-compose.yml");
    if root_compose.exists() {
        return Some(root_compose);
    }

    // Check versioned subdirs (e.g., superset/5/docker-compose.yml)
    if let Ok(entries) = std::fs::read_dir(container_dir) {
        let mut best: Option<(String, PathBuf)> = None;
        for entry in entries.flatten() {
            let subname = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir() && subname.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                let sub_compose = entry.path().join("docker-compose.yml");
                if sub_compose.exists() {
                    // Pick the highest version
                    if best.as_ref().map_or(true, |(prev, _)| subname > *prev) {
                        best = Some((subname, sub_compose));
                    }
                }
            }
        }
        if let Some((_, path)) = best {
            return Some(path);
        }
    }

    None
}

/// Parse a docker-compose.yml file for image tag, ports, dependencies, and credentials.
fn parse_compose_file(
    compose_path: &Path,
    container_name: &str,
) -> (String, String, Vec<(u16, u16)>, Vec<String>, Option<Credentials>, HashMap<String, String>) {
    let default_image = format!("firestream-{}", container_name);
    let content = match std::fs::read_to_string(compose_path) {
        Ok(c) => c,
        Err(_) => return (default_image, "latest".to_string(), vec![], vec![], None, HashMap::new()),
    };

    let yaml: Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (default_image, "latest".to_string(), vec![], vec![], None, HashMap::new()),
    };

    let services = yaml.get("services").and_then(|s| s.as_mapping());
    let mut image_name = default_image.clone();
    let mut image_tag = "latest".to_string();
    let mut ports = Vec::new();
    let mut dependencies = Vec::new();
    let mut credentials = None;
    let env_overrides = HashMap::new();

    if let Some(services_map) = services {
        // Find the primary service (matches container name or has depends_on)
        for (svc_name, svc_config) in services_map {
            let svc_name_str = svc_name.as_str().unwrap_or("");

            // Extract image info from services that reference firestream images
            if let Some(image_str) = svc_config.get("image").and_then(|i| i.as_str()) {
                if image_str.starts_with("firestream-") {
                    // Parse "firestream-airflow:3.0.3" or "firestream-postgresql:${PG_VERSION:-17}"
                    if let Some((name_part, tag_part)) = image_str.split_once(':') {
                        // Check if this is the main service image
                        if name_part == &default_image || svc_name_str == container_name {
                            image_name = name_part.to_string();
                            // Resolve ${VAR:-default} patterns
                            image_tag = resolve_compose_var(tag_part);
                        }

                        // Other firestream images are dependencies
                        if name_part != &default_image {
                            let dep_name = name_part.strip_prefix("firestream-").unwrap_or(name_part);
                            if !dependencies.contains(&dep_name.to_string()) {
                                dependencies.push(dep_name.to_string());
                            }
                        }
                    }
                }
            }

            // Extract ports from the main service
            if svc_name_str == container_name || svc_name_str.contains(container_name) {
                if let Some(port_list) = svc_config.get("ports").and_then(|p| p.as_sequence()) {
                    for port in port_list {
                        if let Some(port_str) = port.as_str() {
                            if let Some((host, container)) = port_str.split_once(':') {
                                if let (Ok(h), Ok(c)) = (host.parse::<u16>(), container.parse::<u16>()) {
                                    ports.push((h, c));
                                }
                            }
                        }
                    }
                }
            }

            // Extract credentials from environment
            if svc_name_str == container_name || svc_name_str.starts_with(&format!("{}-", container_name)) {
                if let Some(env) = svc_config.get("environment") {
                    credentials = extract_credentials(env, container_name);
                }
            }
        }
    }

    (image_name, image_tag, ports, dependencies, credentials, env_overrides)
}

/// Resolve compose variable substitution like ${PG_VERSION:-17}
fn resolve_compose_var(s: &str) -> String {
    if s.starts_with("${") && s.ends_with('}') {
        let inner = &s[2..s.len() - 1];
        if let Some((_var, default)) = inner.split_once(":-") {
            return default.to_string();
        }
    }
    s.to_string()
}

/// Extract credentials from compose environment configuration.
fn extract_credentials(env: &Value, _container_name: &str) -> Option<Credentials> {
    let env_map: HashMap<String, String> = match env {
        Value::Mapping(m) => {
            m.iter()
                .filter_map(|(k, v)| {
                    Some((k.as_str()?.to_string(), v.as_str()?.to_string()))
                })
                .collect()
        }
        Value::Sequence(s) => {
            s.iter()
                .filter_map(|v| {
                    let s = v.as_str()?;
                    let (k, val) = s.split_once('=')?;
                    Some((k.to_string(), val.to_string()))
                })
                .collect()
        }
        _ => return None,
    };

    // Look for common credential patterns
    let username = env_map.iter()
        .find(|(k, _)| k.contains("USERNAME") || k.contains("USER") && !k.contains("HOST"))
        .map(|(_, v)| v.clone());
    let password = env_map.iter()
        .find(|(k, _)| k.contains("PASSWORD"))
        .map(|(_, v)| v.clone());

    if let (Some(user), Some(pass)) = (username, password) {
        Some(Credentials {
            url: String::new(),
            username: user,
            password: pass,
        })
    } else {
        None
    }
}

/// Default manifests when filesystem discovery fails.
/// Mirrors the knowledge from the Makefile.
fn default_manifests(repo_root: &Path) -> Vec<ContainerManifest> {
    let base = repo_root.join("src/containers/firestream");

    let mut manifests = vec![
        ContainerManifest {
            name: "airflow".to_string(),
            nix_package: "airflow".to_string(),
            compose_path: Some(base.join("airflow/docker-compose.yml")),
            compose_working_dir: Some(base.join("airflow")),
            image_name: "firestream-airflow".to_string(),
            image_tag: "3.0.3".to_string(),
            dependencies: vec!["redis".to_string(), "postgresql".to_string()],
            credentials: Some(Credentials { url: "http://localhost:8090".to_string(), username: "airflow".to_string(), password: "airflow".to_string() }),
            ports: vec![(8090, 8080)],
            env_overrides: HashMap::new(),
        },
        ContainerManifest {
            name: "jupyterhub".to_string(),
            nix_package: "jupyterhub".to_string(),
            compose_path: Some(base.join("jupyterhub/docker-compose.yml")),
            compose_working_dir: Some(base.join("jupyterhub")),
            image_name: "firestream-jupyterhub".to_string(),
            image_tag: "5.3.0".to_string(),
            dependencies: vec![],
            credentials: Some(Credentials { url: "http://localhost:8000".to_string(), username: "admin".to_string(), password: "admin123".to_string() }),
            ports: vec![(8000, 8000)],
            env_overrides: HashMap::new(),
        },
        ContainerManifest {
            name: "kafka".to_string(),
            nix_package: "kafka".to_string(),
            compose_path: Some(base.join("kafka/docker-compose.yml")),
            compose_working_dir: Some(base.join("kafka")),
            image_name: "firestream-kafka".to_string(),
            image_tag: "4.0".to_string(),
            dependencies: vec![],
            credentials: None,
            ports: vec![(9092, 9092)],
            env_overrides: HashMap::new(),
        },
        ContainerManifest {
            name: "odoo".to_string(),
            nix_package: "odoo".to_string(),
            compose_path: Some(base.join("odoo/docker-compose.yml")),
            compose_working_dir: Some(base.join("odoo")),
            image_name: "firestream-odoo".to_string(),
            image_tag: "18.0".to_string(),
            dependencies: vec![],
            credentials: Some(Credentials { url: "http://localhost:8069".to_string(), username: "admin@example.com".to_string(), password: "admin".to_string() }),
            ports: vec![(8069, 8069)],
            env_overrides: HashMap::new(),
        },
        ContainerManifest {
            name: "postgresql".to_string(),
            nix_package: "postgresql-17".to_string(),
            compose_path: Some(base.join("postgresql/docker-compose.yml")),
            compose_working_dir: Some(base.join("postgresql")),
            image_name: "firestream-postgresql".to_string(),
            image_tag: "17".to_string(),
            dependencies: vec![],
            credentials: Some(Credentials { url: "localhost:5432".to_string(), username: "firestream".to_string(), password: "firestream".to_string() }),
            ports: vec![(5432, 5432)],
            env_overrides: [("PG_VERSION".to_string(), "17".to_string())].into(),
        },
        ContainerManifest {
            name: "redis".to_string(),
            nix_package: "redis-7".to_string(),
            compose_path: Some(base.join("redis/docker-compose.yml")),
            compose_working_dir: Some(base.join("redis")),
            image_name: "firestream-redis".to_string(),
            image_tag: "7".to_string(),
            dependencies: vec![],
            credentials: None,
            ports: vec![(6379, 6379)],
            env_overrides: HashMap::new(),
        },
        ContainerManifest {
            name: "spark".to_string(),
            nix_package: "spark".to_string(),
            compose_path: Some(base.join("spark/docker-compose.yml")),
            compose_working_dir: Some(base.join("spark")),
            image_name: "firestream-spark".to_string(),
            image_tag: "latest".to_string(),
            dependencies: vec![],
            credentials: None,
            ports: vec![(8080, 8080), (7077, 7077)],
            env_overrides: HashMap::new(),
        },
        ContainerManifest {
            name: "superset".to_string(),
            nix_package: "superset-5".to_string(),
            compose_path: Some(base.join("superset/5/docker-compose.yml")),
            compose_working_dir: Some(base.join("superset/5")),
            image_name: "firestream-superset".to_string(),
            image_tag: "5".to_string(),
            dependencies: vec!["redis".to_string(), "postgresql".to_string()],
            credentials: Some(Credentials { url: "http://localhost:8088".to_string(), username: "admin".to_string(), password: "admin".to_string() }),
            ports: vec![(8088, 8088)],
            env_overrides: HashMap::new(),
        },
    ];

    manifests.sort_by(|a, b| a.name.cmp(&b.name));
    manifests
}

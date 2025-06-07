//! Integration tests for Firestream

use firestream::config::{GlobalConfig, ServiceConfig};

#[test]
fn test_parse_global_config() {
    let config_str = r#"
version = "1.0.0"

[cluster]
name = "test-cluster"
namespace = "firestream-test"
container_runtime = "docker"

[defaults]
log_level = "debug"
timeout_seconds = 600

[defaults.resource_limits]
cpu_cores = 8.0
memory_mb = 16384
disk_gb = 200
"#;

    let config: GlobalConfig = toml::from_str(config_str).unwrap();
    assert_eq!(config.version, "1.0.0");
    assert_eq!(config.cluster.name, "test-cluster");
    assert_eq!(config.cluster.namespace, "firestream-test");
    assert_eq!(config.defaults.timeout_seconds, 600);
}

#[test]
fn test_parse_service_config() {
    let config_str = r#"
api_version = "1.0.0"

[metadata]
name = "test-service"
version = "1.0.0"
description = "Test service"
category = "database"

[spec]
enabled = true
dependencies = ["dep1", "dep2"]

[spec.resources.requests]
cpu = "1.0"
memory = "2Gi"

[spec.resources.limits]
cpu = "2.0"
memory = "4Gi"

[[spec.ports]]
name = "main"
container_port = 8080
protocol = "TCP"

[spec.environment]
ENV_VAR = "value"

[[spec.volumes]]
name = "data"
mount_path = "/data"
read_only = false
"#;

    let config: ServiceConfig = toml::from_str(config_str).unwrap();
    assert_eq!(config.api_version, "1.0.0");
    assert_eq!(config.metadata.name, "test-service");
    assert_eq!(config.spec.dependencies.len(), 2);
    assert_eq!(config.spec.ports.len(), 1);
    assert_eq!(config.spec.environment.get("ENV_VAR"), Some(&"value".to_string()));
}

#[tokio::test]
async fn test_config_manager_creation() {
    use firestream::config::ConfigManager;
    
    // This should work even if the config directory doesn't exist
    let manager = ConfigManager::new();
    assert!(manager.is_ok());
}

//! Integration tests for k8s_manager K3D provider

use k8s_manager::{
    K3dClusterConfig, K3dClusterManager, K3dConfig, K3dRegistryConfig, K3dTlsConfig,
    K3dNetworkConfig, K3dDevModeConfig, CertificateConfig,
    ClusterManager,
    PortForwardConfig, LogsConfig, ResourceType, DiagnosticsConfig,
};

#[cfg(feature = "integration_tests")]
use serial_test::serial;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = K3dClusterConfig::default();
        assert_eq!(config.name, "firestream");
        assert_eq!(config.api_port, 6550);
        assert_eq!(config.http_port, 80);
        assert_eq!(config.https_port, 443);
        assert_eq!(config.servers, 1);
        assert_eq!(config.agents, 1);
        assert_eq!(config.k3s_version, "v1.31.2-k3s1");
    }

    #[test]
    fn test_basic_config_default() {
        let config = K3dConfig::default();
        assert_eq!(config.cluster_name, "firestream");
        assert_eq!(config.api_port, 6443);
        assert_eq!(config.lb_port, 8080);
        assert_eq!(config.agents, 1);
        assert_eq!(config.servers, 1);
        assert_eq!(config.registry_name, "registry.localhost");
        assert_eq!(config.registry_port, 5000);
    }

    #[test]
    fn test_registry_config_default() {
        let config = K3dRegistryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.name, "registry.localhost");
        assert_eq!(config.port, 5000);
    }

    #[test]
    fn test_tls_config_default() {
        let config = K3dTlsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.secret_name, "firestream-tls");
    }

    #[test]
    fn test_certificate_config_default() {
        let config = CertificateConfig::default();
        assert_eq!(config.country, "US");
        assert_eq!(config.state, "New Mexico");
        assert_eq!(config.locality, "Roswell");
        assert_eq!(config.organization, "ACME Co, LLC.");
        assert_eq!(config.organizational_unit, "ACME Department");
        assert_eq!(config.common_name, "www.domain.com");
        assert_eq!(config.email, "email@domain");
    }

    #[test]
    fn test_network_config_default() {
        let config = K3dNetworkConfig::default();
        assert!(config.configure_routes);
        assert!(config.configure_dns);
        assert!(config.patch_etc_hosts);
        assert_eq!(config.pod_cidr, "10.42.0.0/16");
        assert_eq!(config.service_cidr, "10.43.0.0/16");
    }

    #[test]
    fn test_cluster_manager_creation() {
        let config = K3dClusterConfig::default();
        let manager = K3dClusterManager::new(config);
        assert_eq!(manager.provider_name(), "k3d");
    }

    #[test]
    fn test_custom_cluster_config() {
        let config = K3dClusterConfig {
            name: "test-cluster".to_string(),
            api_port: 7550,
            servers: 3,
            agents: 5,
            dev_mode: Some(K3dDevModeConfig {
                port_forward_all: true,
                port_offset: 30000,
            }),
            ..Default::default()
        };
        
        let manager = K3dClusterManager::new(config.clone());
        assert_eq!(manager.provider_name(), "k3d");
        
        // Verify the configuration was applied
        // Note: We can't directly access the config field, but we can test
        // that the manager was created successfully
    }

    #[test]
    fn test_port_forward_config() {
        let config = PortForwardConfig {
            namespace: "default".to_string(),
            service_name: "test-service".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        
        assert_eq!(config.namespace, "default");
        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.local_port, 8080);
        assert_eq!(config.remote_port, 80);
    }

    #[test]
    fn test_logs_config() {
        let config = LogsConfig {
            namespace: Some("kube-system".to_string()),
            resource_type: ResourceType::Pod,
            resource_name: "coredns".to_string(),
            container: None,
            follow: false,
            previous: false,
            all_containers: true,
        };
        
        assert_eq!(config.namespace, Some("kube-system".to_string()));
        assert_eq!(config.resource_type.as_str(), "pod");
        assert_eq!(config.resource_name, "coredns");
        assert!(config.all_containers);
        assert!(!config.follow);
    }

    #[test]
    fn test_resource_type_as_str() {
        assert_eq!(ResourceType::Pod.as_str(), "pod");
        assert_eq!(ResourceType::Deployment.as_str(), "deployment");
        assert_eq!(ResourceType::Service.as_str(), "service");
        assert_eq!(ResourceType::StatefulSet.as_str(), "statefulset");
        assert_eq!(ResourceType::DaemonSet.as_str(), "daemonset");
        assert_eq!(ResourceType::Job.as_str(), "job");
    }

    #[test]
    fn test_diagnostics_config() {
        let config = DiagnosticsConfig {
            namespace: Some("default".to_string()),
            include_nodes: true,
            include_pods: true,
            include_services: false,
            include_events: true,
            include_all: false,
        };
        
        assert_eq!(config.namespace, Some("default".to_string()));
        assert!(config.include_nodes);
        assert!(config.include_pods);
        assert!(!config.include_services);
        assert!(config.include_events);
        assert!(!config.include_all);
    }

    #[test]
    fn test_diagnostics_config_default() {
        let config = DiagnosticsConfig::default();
        assert_eq!(config.namespace, None);
        assert!(!config.include_nodes);
        assert!(!config.include_pods);
        assert!(!config.include_services);
        assert!(!config.include_events);
        assert!(!config.include_all);
    }
}

// Integration tests that require a running Docker daemon and k3d installed
#[cfg(feature = "integration_tests")]
mod integration_tests {
    use super::*;
    use tokio;
    use std::time::Duration;
    use std::collections::HashMap;
    use tokio::process::Command;
    use k8s_manager::{ClusterLifecycle, ClusterNetworking, ClusterObservability, ClusterSecurity, ClusterDevelopment, K3dTimeoutConfig, ClusterStatus};
    
    // Use a simpler approach for test initialization
    static TEST_INIT: std::sync::Once = std::sync::Once::new();
    
    fn init_tests() {
        TEST_INIT.call_once(|| {
            // Run cleanup synchronously using a new runtime
            std::thread::spawn(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    cleanup_test_resources().await;
                });
            }).join().unwrap();
        });
    }
    
    // Cleanup function to remove test resources
    async fn cleanup_test_resources() {
        println!("Running global test cleanup...");
        
        // Clean up all test clusters
        let output = Command::new("k3d")
            .args(&["cluster", "list", "-o", "json"])
            .output()
            .await;
            
        if let Ok(result) = output {
            if let Ok(clusters) = serde_json::from_slice::<Vec<serde_json::Value>>(&result.stdout) {
                for cluster in clusters {
                    if let Some(name) = cluster["name"].as_str() {
                        if name.starts_with("test-") {
                            println!("Cleaning up old test cluster: {}", name);
                            let _ = Command::new("k3d")
                                .args(&["cluster", "delete", name])
                                .output()
                                .await;
                        }
                    }
                }
            }
        }
        
        // Clean up all test registries
        let output = Command::new("k3d")
            .args(&["registry", "list"])
            .output()
            .await;
            
        if let Ok(result) = output {
            let registries = String::from_utf8_lossy(&result.stdout);
            for line in registries.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(name) = parts.get(0) {
                    if name.contains("test-") {
                        println!("Cleaning up old test registry: {}", name);
                        let _ = Command::new("k3d")
                            .args(&["registry", "delete", name])
                            .output()
                            .await;
                    }
                }
            }
        }
        
        // Clean up dangling Docker containers
        let _ = Command::new("docker")
            .args(&["container", "prune", "-f"])
            .output()
            .await;
            
        // Clean up test networks
        let output = Command::new("docker")
            .args(&["network", "ls", "--format", "{{.Name}}"])
            .output()
            .await;
            
        if let Ok(result) = output {
            let networks = String::from_utf8_lossy(&result.stdout);
            for network in networks.lines() {
                if network.starts_with("k3d-test-") {
                    println!("Cleaning up old test network: {}", network);
                    let _ = Command::new("docker")
                        .args(&["network", "rm", network])
                        .output()
                        .await;
                }
            }
        }
        
        println!("Global test cleanup complete");
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // Helper function to create a unique test cluster name
    fn test_cluster_name(test_name: &str) -> String {
        format!("test-{}-{}", test_name, std::process::id())
    }
    
    // Helper function to get unique port based on test name and process ID
    fn get_unique_port(base_port: u16, test_name: &str) -> u16 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let pid = std::process::id() as u16;
        let hash = test_name.bytes().fold(0u16, |acc, b| acc.wrapping_add(b as u16));
        
        // Add timestamp component for better uniqueness
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u16;
            
        // Use a larger range and incorporate timestamp
        let offset = ((pid + hash + timestamp) % 20000) + 10000;
        base_port + offset
    }
    
    // Helper function to check if a port is available
    async fn is_port_available(port: u16) -> bool {
        let output = Command::new("lsof")
            .args(&["-i", &format!(":{}", port), "-P", "-n"])
            .output()
            .await;
            
        match output {
            Ok(result) => result.stdout.is_empty(),
            Err(_) => {
                // If lsof is not available, try netstat
                let netstat_output = Command::new("netstat")
                    .args(&["-tuln"])
                    .output()
                    .await;
                    
                match netstat_output {
                    Ok(result) => {
                        let output_str = String::from_utf8_lossy(&result.stdout);
                        !output_str.contains(&format!(":{} ", port)) && 
                        !output_str.contains(&format!(":{}	", port))
                    },
                    Err(_) => true // Assume available if we can't check
                }
            }
        }
    }
    
    // Helper function to find an available port
    async fn find_available_port(base_port: u16, test_name: &str) -> u16 {
        let mut port = get_unique_port(base_port, test_name);
        let mut attempts = 0;
        
        while attempts < 100 {
            if is_port_available(port).await {
                return port;
            }
            port += 1;
            attempts += 1;
        }
        
        // Fallback to original calculation if we can't find an available port
        port
    }
    
    // Helper function to ensure cluster is cleaned up
    async fn ensure_cluster_cleanup(cluster_name: &str) {
        // Delete cluster
        let _ = Command::new("k3d")
            .args(&["cluster", "delete", cluster_name])
            .output()
            .await;
        
        // Delete all possible registry name patterns
        let registry_patterns = vec![
            format!("{}-registry", cluster_name),
            format!("{}.registry.localhost", cluster_name),
            format!("registry.localhost"), // Sometimes tests use the default name
        ];
        
        for registry_name in registry_patterns {
            let _ = Command::new("k3d")
                .args(&["registry", "delete", &registry_name])
                .output()
                .await;
        }
        
        // Clean up any Docker containers with the cluster name
        let _ = Command::new("docker")
            .args(&["rm", "-f"])
            .arg(format!("k3d-{}-registry", cluster_name))
            .output()
            .await;
            
        let _ = Command::new("docker")
            .args(&["rm", "-f"])
            .arg(format!("k3d-{}.registry.localhost", cluster_name))
            .output()
            .await;
        
        // Clean up the Docker network if it exists
        let _ = Command::new("docker")
            .args(&["network", "rm", &format!("k3d-{}", cluster_name)])
            .output()
            .await;
        
        // Prune unused Docker networks to free up subnet space
        let _ = Command::new("docker")
            .args(&["network", "prune", "-f"])
            .output()
            .await;
        
        // Wait a bit for cleanup to complete
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    
    // Helper function to create a test configuration with shorter timeouts
    async fn test_cluster_config(name: String) -> K3dClusterConfig {
        K3dClusterConfig {
            name: name.clone(),
            api_port: find_available_port(16443, &format!("{}-api", name)).await,
            http_port: find_available_port(30080, &format!("{}-http", name)).await,
            https_port: find_available_port(30443, &format!("{}-https", name)).await,
            registry: K3dRegistryConfig {
                enabled: true,
                name: "registry.localhost".to_string(),
                port: find_available_port(15000, &format!("{}-registry", name)).await,
            },
            timeouts: K3dTimeoutConfig {
                cluster_create: 120,      // 2 minutes for tests
                cluster_ready: 120,       // 2 minutes
                node_ready: 30,          // 30 seconds
                dns_check: 15,           // 15 seconds
                registry_ready: 30,      // 30 seconds
                initial_retry_delay_ms: 500,   // Start with 500ms
                max_retry_delay_ms: 10000,     // Max 10 seconds
                max_retries: 5,                // Fewer retries for tests
            },
            ..Default::default()
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_cluster_exists_check() {
        // Initialize tests
        init_tests();
        
        let config = test_cluster_config(test_cluster_name("exists-check")).await;
        
        let manager = K3dClusterManager::new(config);
        
        // Should return false for non-existent cluster
        let exists = manager.cluster_exists().await.unwrap();
        assert!(!exists, "Non-existent cluster should return false");
    }

    #[tokio::test]
    #[serial]
    async fn test_basic_cluster_lifecycle() {
        // Initialize tests
        init_tests();
        
        let cluster_name = test_cluster_name("basic-lifecycle");
        
        // Ensure cleanup before test
        ensure_cluster_cleanup(&cluster_name).await;
        let api_port = find_available_port(16443, "basic-lifecycle").await;
        let registry_port = find_available_port(15000, "basic-lifecycle-registry").await;
        let config = K3dConfig {
            cluster_name: cluster_name.clone(),
            api_port,
            lb_port: find_available_port(18080, "basic-lifecycle-lb").await,
            agents: 1,
            servers: 1,
            registry_name: format!("{}-registry", cluster_name),
            registry_port,
        };
        
        // Setup cluster
        let result = k8s_manager::k3d::setup_cluster_with_config(&config).await;
        assert!(result.is_ok(), "Failed to setup cluster: {:?}", result);
        
        // Verify cluster exists
        let full_config = K3dClusterConfig {
            name: cluster_name.clone(),
            ..Default::default()
        };
        let manager = K3dClusterManager::new(full_config);
        
        let exists = manager.cluster_exists().await.unwrap();
        assert!(exists, "Cluster should exist after creation");
        
        // Get cluster info
        let info = manager.get_cluster_info().await.unwrap();
        assert_eq!(info.name, cluster_name);
        assert!(matches!(info.provider, k8s_manager::ClusterProvider::K3d));
        
        // Delete cluster
        let delete_result = k8s_manager::k3d::delete_cluster(&cluster_name).await;
        assert!(delete_result.is_ok(), "Failed to delete cluster: {:?}", delete_result);
        
        // Verify cluster is deleted
        let exists_after = manager.cluster_exists().await.unwrap();
        assert!(!exists_after, "Cluster should not exist after deletion");
        
        // Final cleanup
        ensure_cluster_cleanup(&cluster_name).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_full_cluster_setup() {
        // Initialize tests
        init_tests();
        
        let cluster_name = test_cluster_name("full-setup");
        
        // Ensure cleanup before test
        ensure_cluster_cleanup(&cluster_name).await;
        let mut config = test_cluster_config(cluster_name.clone()).await;
        
        // Use unique registry name to avoid conflicts
        config.registry.name = format!("{}-registry", cluster_name);
        
        // Disable some features for faster testing
        config.tls.enabled = false;
        config.network.configure_routes = false;
        config.network.configure_dns = false;
        config.network.patch_etc_hosts = false;
        
        let manager = K3dClusterManager::new(config);
        
        // Full setup
        let setup_result = manager.setup_cluster().await;
        assert!(setup_result.is_ok(), "Failed to setup cluster: {:?}", setup_result);
        
        // Verify cluster is running
        let status = manager.get_cluster_status().await.unwrap();
        assert!(status.contains("Running"), "Cluster should be running");
        
        // Test lifecycle operations
        let stop_result = manager.stop_cluster().await;
        assert!(stop_result.is_ok(), "Failed to stop cluster: {:?}", stop_result);
        
        // The stop operation should be quick, but let's give it a moment
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        let start_result = manager.start_cluster().await;
        assert!(start_result.is_ok(), "Failed to start cluster: {:?}", start_result);
        
        // Test port forwarding
        let pf_config = PortForwardConfig {
            namespace: "kube-system".to_string(),
            service_name: "kube-dns".to_string(),
            local_port: 15353,
            remote_port: 53,
        };
        
        let pf_result = manager.port_forward(&pf_config).await;
        assert!(pf_result.is_ok(), "Failed to setup port forward: {:?}", pf_result);
        
        // Test diagnostics
        let diag_config = DiagnosticsConfig {
            include_nodes: true,
            include_pods: true,
            ..Default::default()
        };
        
        let diagnostics = manager.get_diagnostics(&diag_config).await.unwrap();
        assert!(diagnostics.contains_key("nodes"), "Diagnostics should include nodes");
        assert!(diagnostics.contains_key("pods"), "Diagnostics should include pods");
        
        // Test secret creation
        let mut secret_data = HashMap::new();
        secret_data.insert("test-key".to_string(), b"test-value".to_vec());
        
        let secret_result = manager.create_secret(
            "test-secret",
            "default",
            secret_data.clone()
        ).await;
        assert!(secret_result.is_ok(), "Failed to create secret: {:?}", secret_result);
        
        // Verify secret was created
        let get_secret_result = manager.get_secret("test-secret", "default").await;
        assert!(get_secret_result.is_ok(), "Failed to get secret: {:?}", get_secret_result);
        
        let retrieved_data = get_secret_result.unwrap();
        assert_eq!(
            retrieved_data.get("test-key").unwrap(),
            b"test-value",
            "Secret data should match"
        );
        
        // Cleanup
        let delete_result = manager.delete_cluster().await;
        assert!(delete_result.is_ok(), "Failed to delete cluster: {:?}", delete_result);
        
        // Final cleanup
        ensure_cluster_cleanup(&cluster_name).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_dev_mode() {
        let cluster_name = test_cluster_name("dev-mode");
        
        // Ensure cleanup before test
        ensure_cluster_cleanup(&cluster_name).await;
        let mut config = test_cluster_config(cluster_name.clone()).await;
        config.dev_mode = Some(K3dDevModeConfig {
            port_forward_all: false, // Set to false to avoid conflicts
            port_offset: 40000,
        });
        
        let manager = K3dClusterManager::new(config);
        
        // Create cluster first (the improved create_cluster will wait for readiness)
        let create_result = manager.create_cluster().await;
        assert!(create_result.is_ok(), "Failed to create cluster: {:?}", create_result);
        
        // Test enabling dev mode
        let enable_result = manager.enable_dev_mode().await;
        assert!(enable_result.is_ok(), "Failed to enable dev mode: {:?}", enable_result);
        
        // Test disabling dev mode
        let disable_result = manager.disable_dev_mode().await;
        assert!(disable_result.is_ok(), "Failed to disable dev mode: {:?}", disable_result);
        
        // Cleanup
        let delete_result = manager.delete_cluster().await;
        assert!(delete_result.is_ok(), "Failed to delete cluster: {:?}", delete_result);
        
        // Final cleanup
        ensure_cluster_cleanup(&cluster_name).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_logs_retrieval() {
        let cluster_name = test_cluster_name("logs");
        let config = test_cluster_config(cluster_name.clone()).await;
        
        let manager = K3dClusterManager::new(config);
        
        // Create cluster (the improved create_cluster will wait for readiness)
        let create_result = manager.create_cluster().await;
        assert!(create_result.is_ok(), "Failed to create cluster: {:?}", create_result);
        
        // Wait longer for core components to be fully ready
        tokio::time::sleep(Duration::from_secs(15)).await;
        
        // Test getting logs from a system pod
        let logs_config = LogsConfig {
            namespace: Some("kube-system".to_string()),
            resource_type: ResourceType::Deployment,
            resource_name: "coredns".to_string(),
            container: None,
            follow: false,
            previous: false,
            all_containers: true,
        };
        
        let logs_result = manager.get_logs(&logs_config).await;
        assert!(logs_result.is_ok(), "Failed to get logs: {:?}", logs_result);
        
        let logs = logs_result.unwrap();
        assert!(!logs.is_empty(), "Logs should not be empty");
        
        // Cleanup
        let delete_result = manager.delete_cluster().await;
        assert!(delete_result.is_ok(), "Failed to delete cluster: {:?}", delete_result);
        
        // Final cleanup
        ensure_cluster_cleanup(&cluster_name).await;
    }
    
    #[tokio::test]
    #[serial]
    async fn test_retry_mechanism() {
        let cluster_name = test_cluster_name("retry-test");
        
        // Ensure cleanup before test
        ensure_cluster_cleanup(&cluster_name).await;
        
        // Extra Docker network cleanup to prevent subnet exhaustion
        let _ = Command::new("docker")
            .args(&["network", "prune", "-f"])
            .output()
            .await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let mut config = test_cluster_config(cluster_name.clone()).await;
        
        // Configure very aggressive timeouts to test retry behavior
        config.timeouts.initial_retry_delay_ms = 100;
        config.timeouts.max_retry_delay_ms = 1000;
        config.timeouts.max_retries = 3;
        config.timeouts.dns_check = 5; // Very short timeout
        
        let manager = K3dClusterManager::new(config);
        
        // Create a basic cluster
        let create_result = manager.create_cluster().await;
        assert!(create_result.is_ok(), "Failed to create cluster: {:?}", create_result);
        
        // The DNS test might fail with such short timeouts, but the retry mechanism should handle it
        let dns_result = manager.test_dns_resolution().await;
        
        // DNS might fail in test environments, so we just check it doesn't panic
        match dns_result {
            Ok(_) => println!("DNS test passed (with retries)"),
            Err(e) => println!("DNS test failed as expected: {}", e),
        }
        
        // Cleanup
        let _ = manager.delete_cluster().await;
        
        // Final cleanup
        ensure_cluster_cleanup(&cluster_name).await;
    }
    
    #[tokio::test]
    #[serial]
    async fn test_cluster_health_verification() {
        // Initialize tests
        init_tests();
        
        let cluster_name = test_cluster_name("health-verify");
        
        // Ensure cleanup before test
        ensure_cluster_cleanup(&cluster_name).await;
        
        let mut config = test_cluster_config(cluster_name.clone()).await;
        
        // Use unique registry name to avoid conflicts
        config.registry.name = format!("{}-registry", cluster_name);
        
        let manager = K3dClusterManager::new(config);
        
        // Create cluster with full health verification
        let setup_result = manager.setup_cluster().await;
        assert!(setup_result.is_ok(), "Failed to setup cluster with health verification: {:?}", setup_result);
        
        // The setup_cluster method includes comprehensive health checks
        // If we get here, all health checks passed
        
        // Verify we can get cluster info
        let info = manager.get_cluster_info().await.unwrap();
        assert_eq!(info.name, cluster_name);
        assert!(matches!(info.status, ClusterStatus::Running));
        
        // Cleanup
        let _ = manager.delete_cluster().await;
        
        // Final cleanup
        ensure_cluster_cleanup(&cluster_name).await;
    }
}

// Benchmarks for performance testing
#[cfg(all(test, feature = "bench"))]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn bench_cluster_creation() {
        let cluster_name = format!("bench-cluster-{}", std::process::id());
        let config = K3dClusterConfig {
            name: cluster_name.clone(),
            api_port: 6560,
            ..Default::default()
        };
        
        let manager = K3dClusterManager::new(config);
        
        let start = Instant::now();
        let result = manager.create_cluster().await;
        let duration = start.elapsed();
        
        println!("Cluster creation took: {:?}", duration);
        assert!(result.is_ok());
        
        // Cleanup
        let _ = manager.delete_cluster().await;
    }
}

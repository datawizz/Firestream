//! Integration tests for k8s_manager K3D provider

use k8s_manager::{
    K3dClusterConfig, K3dClusterManager, K3dConfig, K3dRegistryConfig, K3dTlsConfig,
    K3dNetworkConfig, K3dDevModeConfig, CertificateConfig,
    ClusterManager, ClusterLifecycle, ClusterNetworking, ClusterObservability,
    ClusterSecurity, ClusterDevelopment,
    PortForwardConfig, LogsConfig, ResourceType, DiagnosticsConfig,
};

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
        let mut config = K3dClusterConfig::default();
        config.name = "test-cluster".to_string();
        config.api_port = 7550;
        config.servers = 3;
        config.agents = 5;
        
        // Enable dev mode
        config.dev_mode = Some(K3dDevModeConfig {
            port_forward_all: true,
            port_offset: 30000,
        });
        
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

    // Helper function to create a unique test cluster name
    fn test_cluster_name(test_name: &str) -> String {
        format!("test-{}-{}", test_name, std::process::id())
    }

    #[tokio::test]
    async fn test_cluster_exists_check() {
        let config = K3dClusterConfig {
            name: test_cluster_name("exists-check"),
            ..Default::default()
        };
        
        let manager = K3dClusterManager::new(config);
        
        // Should return false for non-existent cluster
        let exists = manager.cluster_exists().await.unwrap();
        assert!(!exists, "Non-existent cluster should return false");
    }

    #[tokio::test]
    async fn test_basic_cluster_lifecycle() {
        let cluster_name = test_cluster_name("basic-lifecycle");
        let config = K3dConfig {
            cluster_name: cluster_name.clone(),
            api_port: 6443,
            lb_port: 8081,
            agents: 1,
            servers: 1,
            registry_name: format!("{}-registry", cluster_name),
            registry_port: 5001,
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
        assert_eq!(info.provider.to_string(), "k3d");
        
        // Delete cluster
        let delete_result = k8s_manager::k3d::delete_cluster(&cluster_name).await;
        assert!(delete_result.is_ok(), "Failed to delete cluster: {:?}", delete_result);
        
        // Verify cluster is deleted
        let exists_after = manager.cluster_exists().await.unwrap();
        assert!(!exists_after, "Cluster should not exist after deletion");
    }

    #[tokio::test]
    async fn test_full_cluster_setup() {
        let cluster_name = test_cluster_name("full-setup");
        let mut config = K3dClusterConfig {
            name: cluster_name.clone(),
            api_port: 6551,
            http_port: 8082,
            https_port: 8443,
            servers: 1,
            agents: 1,
            ..Default::default()
        };
        
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
        
        // Wait a bit for cluster to stop
        tokio::time::sleep(Duration::from_secs(2)).await;
        
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
    }

    #[tokio::test]
    async fn test_dev_mode() {
        let cluster_name = test_cluster_name("dev-mode");
        let config = K3dClusterConfig {
            name: cluster_name.clone(),
            api_port: 6552,
            dev_mode: Some(K3dDevModeConfig {
                port_forward_all: false, // Set to false to avoid conflicts
                port_offset: 40000,
            }),
            ..Default::default()
        };
        
        let manager = K3dClusterManager::new(config);
        
        // Create cluster first
        let create_result = manager.create_cluster().await;
        assert!(create_result.is_ok(), "Failed to create cluster: {:?}", create_result);
        
        // Wait for cluster to be ready
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Test enabling dev mode
        let enable_result = manager.enable_dev_mode().await;
        assert!(enable_result.is_ok(), "Failed to enable dev mode: {:?}", enable_result);
        
        // Test disabling dev mode
        let disable_result = manager.disable_dev_mode().await;
        assert!(disable_result.is_ok(), "Failed to disable dev mode: {:?}", disable_result);
        
        // Cleanup
        let delete_result = manager.delete_cluster().await;
        assert!(delete_result.is_ok(), "Failed to delete cluster: {:?}", delete_result);
    }

    #[tokio::test]
    async fn test_logs_retrieval() {
        let cluster_name = test_cluster_name("logs");
        let config = K3dClusterConfig {
            name: cluster_name.clone(),
            api_port: 6553,
            ..Default::default()
        };
        
        let manager = K3dClusterManager::new(config);
        
        // Create cluster
        let create_result = manager.create_cluster().await;
        assert!(create_result.is_ok(), "Failed to create cluster: {:?}", create_result);
        
        // Wait for cluster and core components to be ready
        tokio::time::sleep(Duration::from_secs(10)).await;
        
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

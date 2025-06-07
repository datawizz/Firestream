//! Trait implementation tests for K3D provider

use k8s_manager::{
    K3dClusterManager, K3dClusterConfig,
    ClusterManager, ClusterLifecycle, ClusterNetworking,
    ClusterObservability, ClusterSecurity, ClusterDevelopment,
    FullClusterManager,
};

#[cfg(test)]
mod trait_tests {
    use super::*;

    /// Verify that K3dClusterManager implements all required traits
    #[test]
    fn test_trait_implementations() {
        let config = K3dClusterConfig::default();
        let manager = K3dClusterManager::new(config);
        
        // These assertions verify at compile time that K3dClusterManager
        // implements all the required traits
        
        // Core trait
        fn assert_cluster_manager<T: ClusterManager>(_: &T) {}
        assert_cluster_manager(&manager);
        
        // Extension traits
        fn assert_lifecycle<T: ClusterLifecycle>(_: &T) {}
        assert_lifecycle(&manager);
        
        fn assert_networking<T: ClusterNetworking>(_: &T) {}
        assert_networking(&manager);
        
        fn assert_observability<T: ClusterObservability>(_: &T) {}
        assert_observability(&manager);
        
        fn assert_security<T: ClusterSecurity>(_: &T) {}
        assert_security(&manager);
        
        fn assert_development<T: ClusterDevelopment>(_: &T) {}
        assert_development(&manager);
        
        // Combined trait
        fn assert_full_manager<T: FullClusterManager>(_: &T) {}
        assert_full_manager(&manager);
    }
    
    /// Test that trait methods are accessible
    #[test]
    fn test_trait_method_accessibility() {
        let config = K3dClusterConfig::default();
        let manager = K3dClusterManager::new(config);
        
        // Test ClusterManager methods are accessible
        assert_eq!(manager.provider_name(), "k3d");
        
        // Note: We can't actually call async methods in a sync test,
        // but we can verify they exist by creating the futures
        let _ = manager.create_cluster();
        let _ = manager.delete_cluster();
        let _ = manager.cluster_exists();
        let _ = manager.get_cluster_info();
        let _ = manager.get_cluster_status();
        let _ = manager.connect_cluster();
        let _ = manager.update_cluster();
        
        // Test ClusterLifecycle methods
        let _ = manager.start_cluster();
        let _ = manager.stop_cluster();
        let _ = manager.restart_cluster();
        
        // Test ClusterNetworking methods
        let pf_config = k8s_manager::PortForwardConfig {
            namespace: "default".to_string(),
            service_name: "test".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        let _ = manager.port_forward(&pf_config);
        let _ = manager.port_forward_all(10000);
        let _ = manager.configure_dns();
        let _ = manager.setup_routes();
        
        // Test ClusterObservability methods
        let logs_config = k8s_manager::LogsConfig {
            namespace: None,
            resource_type: k8s_manager::ResourceType::Pod,
            resource_name: "test".to_string(),
            container: None,
            follow: false,
            previous: false,
            all_containers: false,
        };
        let _ = manager.get_logs(&logs_config);
        let _ = manager.stream_logs(&logs_config);
        
        let diag_config = k8s_manager::DiagnosticsConfig::default();
        let _ = manager.get_diagnostics(&diag_config);
        let _ = manager.get_metrics();
        
        // Test ClusterSecurity methods
        let _ = manager.configure_tls();
        
        use std::collections::HashMap;
        let data = HashMap::new();
        let _ = manager.create_secret("test", "default", data);
        let _ = manager.get_secret("test", "default");
        
        // Test ClusterDevelopment methods
        let _ = manager.enable_dev_mode();
        let _ = manager.disable_dev_mode();
        let _ = manager.setup_registry();
    }
}

/// Mock tests to verify behavior without actually creating clusters
#[cfg(test)]
mod mock_tests {
    use super::*;
    use tokio;
    
    /// Test that provider name is correctly returned
    #[test]
    fn test_provider_name() {
        let config = K3dClusterConfig::default();
        let manager = K3dClusterManager::new(config);
        assert_eq!(manager.provider_name(), "k3d");
    }
    
    /// Test configuration with custom values
    #[test]
    fn test_custom_configuration() {
        let mut config = K3dClusterConfig::default();
        config.name = "custom-cluster".to_string();
        config.api_port = 7000;
        config.servers = 3;
        config.agents = 5;
        
        let manager = K3dClusterManager::new(config);
        // Manager is created successfully with custom config
        assert_eq!(manager.provider_name(), "k3d");
    }
    
    /// Test that update_cluster returns expected error
    #[tokio::test]
    async fn test_update_cluster_not_supported() {
        let config = K3dClusterConfig::default();
        let manager = K3dClusterManager::new(config);
        
        let result = manager.update_cluster().await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("cannot be updated in-place"));
        }
    }
}

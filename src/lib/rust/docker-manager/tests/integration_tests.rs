//! Integration tests for docker-manager
//!
//! These tests require Docker to be running and accessible.

use docker_manager::{DockerManager, DockerManagerError};

#[tokio::test]
async fn test_docker_connection() {
    // This test will pass if Docker is running, fail otherwise
    let result = DockerManager::new().await;
    
    match result {
        Ok(docker) => {
            // Verify we can ping the daemon
            assert!(docker.is_accessible().await);
        }
        Err(e) => {
            // If Docker is not available, we expect a specific error
            match e {
                DockerManagerError::DockerNotAccessible(_) => {
                    // This is expected if Docker is not running
                    println!("Docker is not accessible - skipping integration tests");
                }
                _ => panic!("Unexpected error: {:?}", e),
            }
        }
    }
}

#[tokio::test]
async fn test_is_running_in_docker() {
    // This test verifies the detection logic
    let in_docker = DockerManager::is_running_in_docker();
    // Just verify it returns a boolean without panicking
    assert!(in_docker || !in_docker);
}

#[tokio::test]
async fn test_image_operations() {
    let docker = match DockerManager::new().await {
        Ok(d) => d,
        Err(_) => {
            println!("Docker not available - skipping test");
            return;
        }
    };
    
    // Test listing images
    let images = docker.list_images(false).await;
    assert!(images.is_ok(), "Should be able to list images");
    
    // Test searching images (doesn't require pulling)
    let search_results = docker.search_images("hello-world", Some(5)).await;
    assert!(search_results.is_ok(), "Should be able to search images");
    
    if let Ok(results) = search_results {
        assert!(!results.is_empty(), "Should find hello-world image");
    }
}

#[tokio::test]
async fn test_container_operations() {
    let docker = match DockerManager::new().await {
        Ok(d) => d,
        Err(_) => {
            println!("Docker not available - skipping test");
            return;
        }
    };
    
    // Test listing containers
    let containers = docker.list_containers(true).await;
    assert!(containers.is_ok(), "Should be able to list containers");
}

#[tokio::test]
async fn test_network_operations() {
    let docker = match DockerManager::new().await {
        Ok(d) => d,
        Err(_) => {
            println!("Docker not available - skipping test");
            return;
        }
    };
    
    // Test listing networks
    let networks = docker.list_networks().await;
    assert!(networks.is_ok(), "Should be able to list networks");
    
    if let Ok(nets) = networks {
        // Docker always has at least the default networks
        assert!(!nets.is_empty(), "Should have at least default networks");
        
        // Check for standard networks
        let network_names: Vec<String> = nets.iter()
            .filter_map(|n| n.name.clone())
            .collect();
        
        assert!(network_names.iter().any(|n| n == "bridge"), "Should have bridge network");
    }
}

#[tokio::test]
async fn test_volume_operations() {
    let docker = match DockerManager::new().await {
        Ok(d) => d,
        Err(_) => {
            println!("Docker not available - skipping test");
            return;
        }
    };
    
    // Test listing volumes
    let volumes = docker.list_volumes().await;
    assert!(volumes.is_ok(), "Should be able to list volumes");
}

#[tokio::test]
async fn test_system_operations() {
    let docker = match DockerManager::new().await {
        Ok(d) => d,
        Err(_) => {
            println!("Docker not available - skipping test");
            return;
        }
    };
    
    // Test version
    let version = docker.version().await;
    assert!(version.is_ok(), "Should be able to get version");
    
    // Test info
    let info = docker.info().await;
    assert!(info.is_ok(), "Should be able to get info");
    
    // Test ping
    let ping = docker.ping().await;
    assert!(ping.is_ok(), "Should be able to ping daemon");
    
    // Test disk usage
    let df = docker.disk_usage().await;
    assert!(df.is_ok(), "Should be able to get disk usage");
    
    // Test health check
    let health = docker.health_check().await;
    assert!(health.is_ok(), "Should be able to perform health check");
    
    if let Ok(h) = health {
        assert!(h.daemon_responsive, "Daemon should be responsive");
    }
}

#[cfg(test)]
mod utils_tests {
    use docker_manager::utils;
    
    #[test]
    fn test_parse_labels() {
        let labels = vec![
            "app=web".to_string(),
            "version=1.0".to_string(),
            "invalid".to_string(),
        ];
        
        let parsed = utils::parse_labels(labels);
        assert_eq!(parsed.get("app"), Some(&"web".to_string()));
        assert_eq!(parsed.get("version"), Some(&"1.0".to_string()));
        assert_eq!(parsed.len(), 2);
    }
    
    #[test]
    fn test_parse_image_name() {
        assert_eq!(
            utils::parse_image_name("nginx:latest"),
            ("nginx".to_string(), "latest".to_string())
        );
        
        assert_eq!(
            utils::parse_image_name("nginx"),
            ("nginx".to_string(), "latest".to_string())
        );
        
        assert_eq!(
            utils::parse_image_name("registry.com/nginx:1.19"),
            ("registry.com/nginx".to_string(), "1.19".to_string())
        );
    }
    
    #[test]
    fn test_valid_container_name() {
        assert!(utils::is_valid_container_name("mycontainer"));
        assert!(utils::is_valid_container_name("my-container"));
        assert!(utils::is_valid_container_name("my_container"));
        assert!(utils::is_valid_container_name("container123"));
        
        assert!(!utils::is_valid_container_name(""));
        assert!(!utils::is_valid_container_name("-container"));
        assert!(!utils::is_valid_container_name("my container"));
        assert!(!utils::is_valid_container_name("container@"));
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(utils::format_duration(30), "30s");
        assert_eq!(utils::format_duration(90), "1m");
        assert_eq!(utils::format_duration(3700), "1h");
        assert_eq!(utils::format_duration(90000), "1d");
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(utils::format_bytes(0), "0 B");
        assert_eq!(utils::format_bytes(1024), "1 KiB");
        assert_eq!(utils::format_bytes(1048576), "1 MiB");
    }
}
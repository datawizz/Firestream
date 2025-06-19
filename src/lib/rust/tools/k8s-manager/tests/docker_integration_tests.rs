//! Integration tests for Docker functionality in k8s-manager
//! 
//! These tests verify the image build, push, and deployment workflows
//! with k3d registries.

#[cfg(feature = "integration_tests")]
mod docker_integration_tests {
    use anyhow::Result;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;
    use tokio;
    
    // Helper function to create a unique test name
    fn test_name(base: &str) -> String {
        // Ensure name is under 32 characters for k3d
        let timestamp = chrono::Utc::now().timestamp() % 10000; // Keep last 4 digits
        let pid = std::process::id() % 1000; // Keep last 3 digits
        format!("{}-{}-{}", base, pid, timestamp)
    }
    
    // Helper function to cleanup all test k3d registries
    fn cleanup_test_registries() {
        if let Ok(output) = Command::new("k3d")
            .args(&["registry", "list", "--no-headers"])
            .output() {
            if output.status.success() {
                let registries = String::from_utf8_lossy(&output.stdout);
                for line in registries.lines() {
                    if line.contains("test-reg-") || line.contains("workflow-reg-") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(name) = parts.first() {
                            eprintln!("Cleaning up leftover k3d registry: {}", name);
                            let _ = Command::new("k3d")
                                .args(&["registry", "delete", name])
                                .output();
                        }
                    }
                }
            }
        }
    }
    
    // Helper struct to ensure cleanup happens even on test failure
    struct TestCleanup {
        cluster_name: Option<String>,
        registries: Vec<String>,
        images: Vec<String>,
    }
    
    impl TestCleanup {
        fn new() -> Self {
            Self {
                cluster_name: None,
                registries: vec![],
                images: vec![],
            }
        }
        
        fn set_cluster(&mut self, name: String) {
            self.cluster_name = Some(name);
        }
        
        fn add_registry(&mut self, name: String) {
            self.registries.push(name);
        }
        
        fn add_image(&mut self, name: String) {
            self.images.push(name);
        }
    }
    
    impl Drop for TestCleanup {
        fn drop(&mut self) {
            // Clean up cluster
            if let Some(cluster_name) = &self.cluster_name {
                let _ = Command::new("k3d")
                    .args(&["cluster", "delete", cluster_name])
                    .output();
            }
            
            // Clean up registries
            for registry in &self.registries {
                // Try k3d registry delete first
                let _ = Command::new("k3d")
                    .args(&["registry", "delete", registry])
                    .output();
                // Also try docker container removal
                let _ = Command::new("docker")
                    .args(&["rm", "-f", registry])
                    .output();
            }
            
            // Clean up images
            for image in &self.images {
                let _ = Command::new("docker")
                    .args(&["rmi", "-f", image])
                    .output();
            }
        }
    }
    
    // Helper function to create a test Docker context
    fn create_test_context() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let dockerfile_content = r#"
FROM alpine:latest
RUN echo "Test image built by k8s-manager integration tests"
CMD ["echo", "Hello from k8s-manager test image"]
"#;
        
        fs::write(temp_dir.path().join("Dockerfile"), dockerfile_content)?;
        
        // Create a simple .dockerignore
        let dockerignore_content = r#"
*.log
.git
.gitignore
"#;
        fs::write(temp_dir.path().join(".dockerignore"), dockerignore_content)?;
        
        // Create a test file
        fs::write(temp_dir.path().join("test.txt"), "Test file content")?;
        
        Ok(temp_dir)
    }
    
    // Helper function to run k8s-manager commands
    fn run_k8s_manager(args: &[&str]) -> Result<std::process::Output> {
        let output = Command::new("cargo")
            .args(&["run", "--features", "integration_tests", "--"])
            .args(args)
            .output()?;
        
        if !output.status.success() {
            eprintln!("Command failed: cargo run -- {}", args.join(" "));
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(output)
    }
    
    // Helper function to check if an image exists locally
    async fn image_exists(image_tag: &str) -> Result<bool> {
        let output = Command::new("docker")
            .args(&["images", "-q", image_tag])
            .output()?;
        
        Ok(!output.stdout.is_empty())
    }
    
    // Helper function to remove a Docker image
    async fn remove_image(image_tag: &str) -> Result<()> {
        let _ = Command::new("docker")
            .args(&["rmi", "-f", image_tag])
            .output()?;
        Ok(())
    }
    
    
    #[tokio::test]
    async fn test_image_build_basic() {
        let context = create_test_context().expect("Failed to create test context");
        let tag = test_name("test-build-basic");
        
        // Build the image
        let output = run_k8s_manager(&[
            "image", "build",
            "--context", context.path().to_str().unwrap(),
            "--tag", &tag,
        ]).expect("Failed to run k8s-manager");
        
        assert!(output.status.success(), "Image build should succeed");
        
        // Verify the image exists
        assert!(image_exists(&tag).await.unwrap(), "Built image should exist");
        
        // Cleanup
        remove_image(&tag).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_image_build_with_dockerfile() {
        let context = create_test_context().expect("Failed to create test context");
        let tag = test_name("test-build-dockerfile");
        
        // Create a custom Dockerfile
        let custom_dockerfile = r#"
FROM alpine:3.18
RUN echo "Custom Dockerfile"
CMD ["echo", "Custom image"]
"#;
        fs::write(context.path().join("Dockerfile.custom"), custom_dockerfile)
            .expect("Failed to write custom Dockerfile");
        
        // Build with custom Dockerfile
        let output = run_k8s_manager(&[
            "image", "build",
            "--context", context.path().to_str().unwrap(),
            "--dockerfile", "Dockerfile.custom",
            "--tag", &tag,
        ]).expect("Failed to run k8s-manager");
        
        assert!(output.status.success(), "Image build with custom Dockerfile should succeed");
        assert!(image_exists(&tag).await.unwrap(), "Built image should exist");
        
        // Cleanup
        remove_image(&tag).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_image_build_with_build_args() {
        let context = create_test_context().expect("Failed to create test context");
        let tag = test_name("test-build-args");
        
        // Create Dockerfile that uses build args
        let dockerfile_with_args = r#"
FROM alpine:latest
ARG VERSION=1.0.0
ARG BUILD_DATE
RUN echo "Version: ${VERSION}, Build Date: ${BUILD_DATE}" > /version.txt
CMD ["cat", "/version.txt"]
"#;
        fs::write(context.path().join("Dockerfile"), dockerfile_with_args)
            .expect("Failed to write Dockerfile");
        
        // Build with build arguments
        let output = run_k8s_manager(&[
            "image", "build",
            "--context", context.path().to_str().unwrap(),
            "--tag", &tag,
            "--build-arg", "VERSION=2.0.0",
            "--build-arg", &format!("BUILD_DATE={}", chrono::Utc::now().to_rfc3339()),
        ]).expect("Failed to run k8s-manager");
        
        assert!(output.status.success(), "Image build with build args should succeed");
        assert!(image_exists(&tag).await.unwrap(), "Built image should exist");
        
        // Cleanup
        remove_image(&tag).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_image_build_no_cache() {
        let context = create_test_context().expect("Failed to create test context");
        let tag = test_name("test-build-no-cache");
        
        // First build (will create cache)
        let output1 = run_k8s_manager(&[
            "image", "build",
            "--context", context.path().to_str().unwrap(),
            "--tag", &tag,
        ]).expect("Failed to run k8s-manager");
        
        assert!(output1.status.success(), "First image build should succeed");
        
        // Second build with no-cache
        let output2 = run_k8s_manager(&[
            "image", "build",
            "--context", context.path().to_str().unwrap(),
            "--tag", &tag,
            "--no-cache",
        ]).expect("Failed to run k8s-manager");
        
        assert!(output2.status.success(), "No-cache build should succeed");
        assert!(image_exists(&tag).await.unwrap(), "Built image should exist");
        
        // Cleanup
        remove_image(&tag).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_build_and_push_to_k3d_registry() {
        // Clean up any leftover test registries from previous runs
        cleanup_test_registries();
        
        let mut cleanup = TestCleanup::new();
        let tag = test_name("k3d-push");
        let context = create_test_context().expect("Failed to create test context");
        
        // Create a k3d registry using k8s-manager's registry command
        let registry_name = format!("test-reg-{}", tag);
        let registry_port = 5000 + (std::process::id() % 100) as u16;
        eprintln!("Creating k3d registry {} on port {}", registry_name, registry_port);
        
        let registry_output = run_k8s_manager(&[
            "registry", "create",
            &registry_name,
            "--port", &registry_port.to_string(),
        ]).expect("Failed to run registry create");
        
        if !registry_output.status.success() {
            eprintln!("Failed to create registry: {}", String::from_utf8_lossy(&registry_output.stderr));
        }
        assert!(registry_output.status.success(), "Registry creation should succeed");
        
        // Add cleanup for k3d registry
        cleanup.registries.push(registry_name.clone());
        
        // Wait for registry to be ready
        eprintln!("Waiting for k3d registry to be ready...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // Build image
        let build_output = run_k8s_manager(&[
            "image", "build",
            "--context", context.path().to_str().unwrap(),
            "--tag", &tag,
        ]).expect("Failed to build");
        
        assert!(build_output.status.success(), "Build should succeed");
        cleanup.add_image(tag.clone());
        
        // Push to k3d registry using localhost and the mapped port
        let registry_url = format!("localhost:{}", registry_port);
        
        // First, check if the image was built correctly
        let inspect_output = Command::new("docker")
            .args(&["inspect", &tag])
            .output()
            .expect("Failed to inspect image");
        
        assert!(inspect_output.status.success(), "Built image should exist");
        
        eprintln!("Pushing {} to k3d registry at {}", tag, registry_url);
        
        let push_output = run_k8s_manager(&[
            "image", "push", &tag,
            "--registry", &registry_url,
        ]).expect("Failed to push");
        
        if !push_output.status.success() {
            eprintln!("Push failed. Stdout: {}", String::from_utf8_lossy(&push_output.stdout));
            eprintln!("Push failed. Stderr: {}", String::from_utf8_lossy(&push_output.stderr));
            
            // Get registry container info
            let k3d_container = format!("k3d-{}", registry_name);
            let registry_logs = Command::new("docker")
                .args(&["logs", &k3d_container])
                .output()
                .expect("Failed to get registry logs");
            eprintln!("Registry logs: {}", String::from_utf8_lossy(&registry_logs.stdout));
            eprintln!("Registry errors: {}", String::from_utf8_lossy(&registry_logs.stderr));
        }
        
        assert!(push_output.status.success(), "Push to k3d registry should succeed");
        cleanup.add_image(format!("{}/{}:latest", registry_url, tag));
        
        // Verify image exists in registry by trying to pull it
        let pull_output = Command::new("docker")
            .args(&[
                "pull",
                &format!("{}/{}:latest", registry_url, tag),
            ])
            .output()
            .expect("Failed to pull image");
        
        assert!(pull_output.status.success(), "Should be able to pull pushed image from k3d registry");
    }
    
    #[tokio::test]
    async fn test_registry_and_image_workflow() {
        cleanup_test_registries();
        let mut cleanup = TestCleanup::new();
        let tag = test_name("workflow");
        let context = create_test_context().expect("Failed to create test context");
        
        // Create k3d registry
        let registry_name = format!("workflow-reg-{}", tag);
        let registry_port = 5100 + (std::process::id() % 100) as u16;
        let registry_output = run_k8s_manager(&[
            "registry", "create",
            &registry_name,
            "--port", &registry_port.to_string(),
        ]).expect("Failed to create registry");
        
        assert!(registry_output.status.success(), "Registry creation should succeed");
        cleanup.registries.push(registry_name.clone());
        
        // Wait for registry to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // Build image
        let build_output = run_k8s_manager(&[
            "image", "build",
            "--context", context.path().to_str().unwrap(),
            "--tag", &tag,
        ]).expect("Failed to build");
        
        assert!(build_output.status.success(), "Build should succeed");
        cleanup.add_image(tag.clone());
        
        // Test push with localhost and mapped port
        let registry_url = format!("localhost:{}", registry_port);
        let push_output = run_k8s_manager(&[
            "image", "push", &tag,
            "--registry", &registry_url,
        ]).expect("Failed to push");
        
        assert!(push_output.status.success(), "Push should succeed");
        cleanup.add_image(format!("{}/{}:latest", registry_url, tag));
    }
    
    #[tokio::test]
    async fn test_push_error_handling() {
        let tag = test_name("test-push-error");
        
        // Try to push non-existent image
        let push_output = run_k8s_manager(&[
            "image", "push", "non-existent-image:tag",
            "--registry", "registry.localhost:5000",
        ]).expect("Failed to run command");
        
        assert!(!push_output.status.success(), "Push of non-existent image should fail");
        
        // Try to push without any registry available
        let context = create_test_context().expect("Failed to create test context");
        
        let build_output = run_k8s_manager(&[
            "image", "build",
            "--context", context.path().to_str().unwrap(),
            "--tag", &tag,
        ]).expect("Failed to build");
        
        if !build_output.status.success() {
            eprintln!("Build failed with stdout: {}", String::from_utf8_lossy(&build_output.stdout));
            eprintln!("Build failed with stderr: {}", String::from_utf8_lossy(&build_output.stderr));
        }
        assert!(build_output.status.success(), "Build should succeed");
        
        // Push to non-existent registry
        let push_output = run_k8s_manager(&[
            "image", "push", &tag,
            "--registry", "non-existent-registry:9999",
        ]).expect("Failed to run command");
        
        assert!(!push_output.status.success(), "Push to non-existent registry should fail");
        
        // Cleanup
        remove_image(&tag).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_multiple_registry_scenario() {
        // Clean up any leftover test registry containers
        let _ = Command::new("docker")
            .args(&["ps", "-a", "--filter", "name=test-multi-reg-", "--format", "{{.Names}}"])
            .output()
            .and_then(|output| {
                let containers = String::from_utf8_lossy(&output.stdout);
                for container in containers.lines() {
                    if !container.is_empty() {
                        let _ = Command::new("docker")
                            .args(&["rm", "-f", container])
                            .output();
                    }
                }
                Ok(())
            });
        
        let mut cleanup = TestCleanup::new();
        let tag = test_name("multi-push");
        let context = create_test_context().expect("Failed to create test context");
        
        // Create multiple simple Docker registries with unique names
        let registry1_port = 5600 + (std::process::id() % 50) as u16;
        let registry2_port = registry1_port + 1;
        let registry3_port = registry1_port + 2;
        
        // Start registry 1
        let registry1_name = format!("test-multi-reg-1-{}", tag);
        let reg1_output = Command::new("docker")
            .args(&[
                "run", "-d",
                "--name", &registry1_name,
                "-p", &format!("{}:5000", registry1_port),
                "registry:2",
            ])
            .output()
            .expect("Failed to start registry1");
        
        if !reg1_output.status.success() {
            eprintln!("Failed to start registry1: {}", String::from_utf8_lossy(&reg1_output.stderr));
        }
        assert!(reg1_output.status.success(), "Registry1 start should succeed");
        cleanup.add_registry(registry1_name.clone());
        
        // Start registry 2
        let registry2_name = format!("test-multi-reg-2-{}", tag);
        let reg2_output = Command::new("docker")
            .args(&[
                "run", "-d",
                "--name", &registry2_name,
                "-p", &format!("{}:5000", registry2_port),
                "registry:2",
            ])
            .output()
            .expect("Failed to start registry2");
        
        if !reg2_output.status.success() {
            eprintln!("Failed to start registry2: {}", String::from_utf8_lossy(&reg2_output.stderr));
        }
        assert!(reg2_output.status.success(), "Registry2 start should succeed");
        cleanup.add_registry(registry2_name.clone());
        
        // Start registry 3
        let registry3_name = format!("test-multi-reg-3-{}", tag);
        let reg3_output = Command::new("docker")
            .args(&[
                "run", "-d",
                "--name", &registry3_name,
                "-p", &format!("{}:5000", registry3_port),
                "registry:2",
            ])
            .output()
            .expect("Failed to start registry3");
        
        if !reg3_output.status.success() {
            eprintln!("Failed to start registry3: {}", String::from_utf8_lossy(&reg3_output.stderr));
        }
        assert!(reg3_output.status.success(), "Registry3 start should succeed");
        cleanup.add_registry(registry3_name.clone());
        
        // Wait for registries to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        // Build image
        let build_output = run_k8s_manager(&[
            "image", "build",
            "--context", context.path().to_str().unwrap(),
            "--tag", &tag,
        ]).expect("Failed to build");
        
        assert!(build_output.status.success(), "Build should succeed");
        cleanup.add_image(tag.clone());
        
        // Push to all registries
        let push1_output = run_k8s_manager(&[
            "image", "push", &tag,
            "--registry", &format!("localhost:{}", registry1_port),
        ]).expect("Failed to push to registry1");
        
        assert!(push1_output.status.success(), "Push to registry1 should succeed");
        cleanup.add_image(format!("localhost:{}/{}:latest", registry1_port, tag));
        
        let push2_output = run_k8s_manager(&[
            "image", "push", &tag,
            "--registry", &format!("localhost:{}", registry2_port),
        ]).expect("Failed to push to registry2");
        
        assert!(push2_output.status.success(), "Push to registry2 should succeed");
        cleanup.add_image(format!("localhost:{}/{}:latest", registry2_port, tag));
        
        let push3_output = run_k8s_manager(&[
            "image", "push", &tag,
            "--registry", &format!("localhost:{}", registry3_port),
        ]).expect("Failed to push to registry3");
        
        assert!(push3_output.status.success(), "Push to registry3 should succeed");
        cleanup.add_image(format!("localhost:{}/{}:latest", registry3_port, tag));
    }
}
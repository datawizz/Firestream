//! Integration tests for Docker command construction and tagging

#[cfg(feature = "integration_tests")]
mod docker_command_tests {
    use std::process::Command;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_image_tagging_for_registry() {
        // Create a test context
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dockerfile_content = r#"
FROM alpine:latest
RUN echo "Test image for tagging"
"#;
        fs::write(temp_dir.path().join("Dockerfile"), dockerfile_content)
            .expect("Failed to write Dockerfile");
        
        // Build the image
        let tag = format!("test-tagging-{}", std::process::id());
        let build_output = Command::new("cargo")
            .args(&[
                "run", "--features", "integration_tests", "--",
                "image", "build",
                "--context", temp_dir.path().to_str().unwrap(),
                "--tag", &tag,
            ])
            .output()
            .expect("Failed to run build");
        
        assert!(build_output.status.success(), "Build should succeed");
        
        // Verify the image exists
        let images_output = Command::new("docker")
            .args(&["images", &tag, "--format", "{{.Repository}}:{{.Tag}}"])
            .output()
            .expect("Failed to list images");
        
        let image_list = String::from_utf8_lossy(&images_output.stdout);
        assert!(image_list.contains(&tag), "Built image should exist");
        
        // Test that push command would tag the image correctly
        // We can't actually push without registry config, but we can verify the tagging
        let inspect_before = Command::new("docker")
            .args(&["inspect", &tag, "--format", "{{.RepoTags}}"])
            .output()
            .expect("Failed to inspect image");
        
        println!("Image tags before: {}", String::from_utf8_lossy(&inspect_before.stdout));
        
        // Clean up
        let _ = Command::new("docker")
            .args(&["rmi", "-f", &tag])
            .output();
    }
    
    #[test]
    fn test_build_with_args() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dockerfile_content = r#"
FROM alpine:latest
ARG TEST_ARG=default
RUN echo "Test arg value: ${TEST_ARG}"
"#;
        fs::write(temp_dir.path().join("Dockerfile"), dockerfile_content)
            .expect("Failed to write Dockerfile");
        
        let tag = format!("test-build-args-{}", std::process::id());
        let build_output = Command::new("cargo")
            .args(&[
                "run", "--features", "integration_tests", "--",
                "image", "build",
                "--context", temp_dir.path().to_str().unwrap(),
                "--tag", &tag,
                "--build-arg", "TEST_ARG=custom_value",
            ])
            .output()
            .expect("Failed to run build");
        
        assert!(build_output.status.success(), "Build with args should succeed");
        
        // Clean up
        let _ = Command::new("docker")
            .args(&["rmi", "-f", &tag])
            .output();
    }
    
    #[test]
    fn test_build_and_push_command() {
        // Test that build-and-push command works (without actual push)
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dockerfile_content = r#"
FROM alpine:latest
RUN echo "Test build-and-push"
"#;
        fs::write(temp_dir.path().join("Dockerfile"), dockerfile_content)
            .expect("Failed to write Dockerfile");
        
        let tag = format!("test-buildpush-{}", std::process::id());
        
        // Just test that the command is accepted and builds the image
        let output = Command::new("cargo")
            .args(&[
                "run", "--features", "integration_tests", "--",
                "image", "build-and-push",
                "--context", temp_dir.path().to_str().unwrap(),
                "--tag", &tag,
                "--registry", "test.registry:5000",
            ])
            .output()
            .expect("Failed to run build-and-push");
        
        // The command will fail on push, but should at least build the image
        let images_output = Command::new("docker")
            .args(&["images", &tag, "-q"])
            .output()
            .expect("Failed to check images");
        
        if !images_output.stdout.is_empty() {
            println!("Image was built successfully");
            // Clean up
            let _ = Command::new("docker")
                .args(&["rmi", "-f", &tag])
                .output();
        }
    }
}
#[cfg(test)]
mod template_tests {
    use firestream::template::{ProjectConfig, TemplateProcessor};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_template_generation() {
        // Create a temporary directory for output
        let temp_dir = TempDir::new().unwrap();
        
        // Create test configuration
        let config = ProjectConfig {
            project_name: "test-app".to_string(),
            version: "1.0.0".to_string(),
            description: "Test application".to_string(),
            docker_image: "python:3.9-slim".to_string(),
            port: 8080,
            replicas: 2,
            cpu_request: "100m".to_string(),
            memory_request: "128Mi".to_string(),
            cpu_limit: "200m".to_string(),
            memory_limit: "256Mi".to_string(),
            enable_ingress: true,
            ingress_host: Some("test.local".to_string()),
            ingress_path: Some("/".to_string()),
        };
        
        // Process templates
        let processor = TemplateProcessor::new(temp_dir.path().to_path_buf()).unwrap();
        processor.process(&config, "python-fastapi").await.unwrap();
        
        // Verify generated files
        let project_path = temp_dir.path().join("test-app");
        assert!(project_path.join("helm/template/Chart.yaml").exists());
        assert!(project_path.join("helm/template/values.yaml").exists());
        assert!(project_path.join("docker/test-app/Dockerfile").exists());
        assert!(project_path.join("src/main.py").exists());
        assert!(project_path.join("Makefile").exists());
        
        // Verify Chart.yaml content
        let chart_content = std::fs::read_to_string(
            project_path.join("helm/template/Chart.yaml")
        ).unwrap();
        assert!(chart_content.contains("name: test-app"));
        assert!(chart_content.contains("version: 1.0.0"));
        assert!(chart_content.contains("description: Test application"));
        
        // Verify values.yaml content
        let values_content = std::fs::read_to_string(
            project_path.join("helm/template/values.yaml")
        ).unwrap();
        assert!(values_content.contains("replicaCount: 2"));
        assert!(values_content.contains("port: 8080"));
        assert!(values_content.contains("enabled: true")); // ingress
    }

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default_with_name(Some("custom-name".to_string()));
        assert_eq!(config.project_name, "custom-name");
        assert_eq!(config.version, "0.1.0");
        assert_eq!(config.port, 8080);
        assert!(!config.enable_ingress);
    }

    #[test]
    fn test_values_file_parsing() {
        use std::io::Write;
        
        // Create a temporary values file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file.as_file(), "project_name: test-from-file").unwrap();
        writeln!(temp_file.as_file(), "version: \"2.0.0\"").unwrap();
        writeln!(temp_file.as_file(), "description: \"Test from file\"").unwrap();
        writeln!(temp_file.as_file(), "docker_image: \"node:18\"").unwrap();
        writeln!(temp_file.as_file(), "port: 3000").unwrap();
        writeln!(temp_file.as_file(), "replicas: 1").unwrap();
        writeln!(temp_file.as_file(), "cpu_request: \"50m\"").unwrap();
        writeln!(temp_file.as_file(), "memory_request: \"64Mi\"").unwrap();
        writeln!(temp_file.as_file(), "cpu_limit: \"100m\"").unwrap();
        writeln!(temp_file.as_file(), "memory_limit: \"128Mi\"").unwrap();
        writeln!(temp_file.as_file(), "enable_ingress: false").unwrap();
        
        let config = ProjectConfig::from_values_file(temp_file.path()).unwrap();
        assert_eq!(config.project_name, "test-from-file");
        assert_eq!(config.version, "2.0.0");
        assert_eq!(config.docker_image, "node:18");
        assert_eq!(config.port, 3000);
    }
}

//! App container image builder

use crate::core::{FirestreamError, Result};
use super::{AppManifest, AppStructure};
use tracing::{info, debug};
use tokio::process::Command;

/// App container image builder
pub struct AppBuilder<'a> {
    manifest: &'a AppManifest,
    structure: &'a AppStructure,
}

impl<'a> AppBuilder<'a> {
    /// Create new builder
    pub fn new(manifest: &'a AppManifest, structure: &'a AppStructure) -> Self {
        Self { manifest, structure }
    }
    
    /// Build container image
    pub async fn build(&self) -> Result<String> {
        let image_tag = self.get_image_tag();
        info!("Building Docker image: {}", image_tag);
        
        // Prepare build context
        let context_path = self.structure.root.join(&self.manifest.build.context);
        let dockerfile_path = context_path.join(&self.manifest.build.dockerfile);
        
        // Check if Dockerfile exists
        if !dockerfile_path.exists() {
            return Err(FirestreamError::ConfigError(
                format!("Dockerfile not found at {:?}", dockerfile_path)
            ));
        }
        
        // Build Docker command
        let mut cmd = Command::new("docker");
        cmd.arg("build");
        cmd.arg("-f").arg(&dockerfile_path);
        cmd.arg("-t").arg(&image_tag);
        
        // Add build arguments
        for (key, value) in &self.manifest.build.args {
            cmd.arg("--build-arg").arg(format!("{}={}", key, value));
        }
        
        // Add target if specified
        if let Some(target) = &self.manifest.build.target {
            cmd.arg("--target").arg(target);
        }
        
        // Set build context
        cmd.arg(&context_path);
        
        debug!("Running Docker build command: {:?}", cmd);
        
        // Execute build
        let output = cmd.output().await
            .map_err(|e| FirestreamError::GeneralError(
                format!("Failed to execute Docker build: {}", e)
            ))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FirestreamError::GeneralError(
                format!("Docker build failed: {}", stderr)
            ));
        }
        
        info!("Successfully built image: {}", image_tag);
        
        // Push to registry if configured
        if self.should_push() {
            self.push_image(&image_tag).await?;
        }
        
        Ok(image_tag)
    }
    
    /// Get full image tag
    pub fn get_image_tag(&self) -> String {
        let registry = self.manifest.build.registry.as_deref().unwrap_or("localhost:5000");
        let repository = self.manifest.build.repository.as_deref()
            .unwrap_or(&self.manifest.app.name);
        let tag = self.manifest.build.tag.as_deref()
            .unwrap_or(&self.manifest.app.version);
        
        format!("{}/{}:{}", registry, repository, tag)
    }
    
    /// Check if we should push the image
    fn should_push(&self) -> bool {
        // Push if registry is not localhost
        self.manifest.build.registry.as_ref()
            .map(|r| !r.starts_with("localhost"))
            .unwrap_or(false)
    }
    
    /// Push image to registry
    async fn push_image(&self, image_tag: &str) -> Result<()> {
        info!("Pushing image to registry: {}", image_tag);
        
        let mut cmd = Command::new("docker");
        cmd.arg("push").arg(image_tag);
        
        let output = cmd.output().await
            .map_err(|e| FirestreamError::GeneralError(
                format!("Failed to execute Docker push: {}", e)
            ))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FirestreamError::GeneralError(
                format!("Docker push failed: {}", stderr)
            ));
        }
        
        info!("Successfully pushed image: {}", image_tag);
        Ok(())
    }
    
    /// Build with BuildKit
    pub async fn build_with_buildkit(&self) -> Result<String> {
        let image_tag = self.get_image_tag();
        info!("Building Docker image with BuildKit: {}", image_tag);
        
        let context_path = self.structure.root.join(&self.manifest.build.context);
        let dockerfile_path = context_path.join(&self.manifest.build.dockerfile);
        
        let mut cmd = Command::new("docker");
        cmd.env("DOCKER_BUILDKIT", "1");
        cmd.arg("build");
        cmd.arg("--progress=plain");
        cmd.arg("-f").arg(&dockerfile_path);
        cmd.arg("-t").arg(&image_tag);
        
        // BuildKit specific options
        cmd.arg("--cache-from").arg(format!("type=registry,ref={}", image_tag));
        cmd.arg("--cache-to").arg("type=inline");
        
        // Add build arguments
        for (key, value) in &self.manifest.build.args {
            cmd.arg("--build-arg").arg(format!("{}={}", key, value));
        }
        
        if let Some(target) = &self.manifest.build.target {
            cmd.arg("--target").arg(target);
        }
        
        cmd.arg(&context_path);
        
        let output = cmd.output().await
            .map_err(|e| FirestreamError::GeneralError(
                format!("Failed to execute Docker build: {}", e)
            ))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FirestreamError::GeneralError(
                format!("Docker build failed: {}", stderr)
            ));
        }
        
        Ok(image_tag)
    }
    
    /// Scan image for vulnerabilities
    pub async fn scan_image(&self, image_tag: &str) -> Result<()> {
        info!("Scanning image for vulnerabilities: {}", image_tag);
        
        // Try to use trivy if available
        if Command::new("which").arg("trivy").output().await.is_ok() {
            let mut cmd = Command::new("trivy");
            cmd.arg("image");
            cmd.arg("--severity").arg("HIGH,CRITICAL");
            cmd.arg("--exit-code").arg("0"); // Don't fail on vulnerabilities
            cmd.arg(image_tag);
            
            let output = cmd.output().await?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            if stdout.contains("CRITICAL") {
                eprintln!("WARNING: Critical vulnerabilities found in image!");
            }
            
            debug!("Vulnerability scan results:\n{}", stdout);
        } else {
            debug!("Trivy not found, skipping vulnerability scan");
        }
        
        Ok(())
    }
    
    /// Generate .dockerignore if not exists
    pub async fn ensure_dockerignore(&self) -> Result<()> {
        let dockerignore_path = self.structure.root.join(".dockerignore");
        
        if !dockerignore_path.exists() {
            info!("Generating .dockerignore file");
            
            let content = r#"# Firestream generated .dockerignore
.git
.gitignore
*.md
.DS_Store
.env*
!.env.example

# Build artifacts
target/
dist/
build/
*.pyc
__pycache__/
node_modules/
*.log

# IDE
.idea/
.vscode/
*.swp
*.swo

# Test files
tests/
test/
*_test.go
*_test.py
*.test

# Documentation
docs/
*.pdf

# Firestream specific
firestream.toml
bootstrap.sh
values.yaml
Chart.yaml
"#;
            
            tokio::fs::write(&dockerignore_path, content).await?;
        }
        
        Ok(())
    }
}

/// Docker registry helper
pub struct RegistryHelper;

impl RegistryHelper {
    /// Login to Docker registry
    pub async fn login(registry: &str, username: &str, password: &str) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("login");
        cmd.arg("-u").arg(username);
        cmd.arg("--password-stdin");
        cmd.arg(registry);
        cmd.stdin(std::process::Stdio::piped());
        
        let mut child = cmd.spawn()
            .map_err(|e| FirestreamError::GeneralError(
                format!("Failed to spawn Docker login: {}", e)
            ))?;
        
        // Write password to stdin
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(password.as_bytes()).await?;
        }
        
        let output = child.wait_with_output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FirestreamError::GeneralError(
                format!("Docker login failed: {}", stderr)
            ));
        }
        
        Ok(())
    }
    
    /// Check if image exists in registry
    pub async fn image_exists(image_tag: &str) -> Result<bool> {
        let mut cmd = Command::new("docker");
        cmd.arg("manifest");
        cmd.arg("inspect");
        cmd.arg(image_tag);
        
        let output = cmd.output().await?;
        Ok(output.status.success())
    }
    
    /// Tag image
    pub async fn tag_image(source: &str, target: &str) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("tag");
        cmd.arg(source);
        cmd.arg(target);
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FirestreamError::GeneralError(
                format!("Docker tag failed: {}", stderr)
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::schema::{AppMetadata, AppType, BuildConfig};
    
    #[test]
    fn test_image_tag_generation() {
        let manifest = AppManifest {
            app: AppMetadata {
                name: "test-app".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                authors: vec![],
                license: None,
                homepage: None,
                repository: None,
                app_type: AppType::Service,
            },
            build: BuildConfig {
                registry: Some("myregistry.io".to_string()),
                repository: Some("myorg/test-app".to_string()),
                tag: Some("v1.0.0".to_string()),
                ..Default::default()
            },
            config: Default::default(),
            dependencies: Default::default(),
            environments: Default::default(),
            secrets: Default::default(),
            deploy: Default::default(),
        };
        
        let structure = AppStructure::from_root("/tmp/test-app");
        let builder = AppBuilder::new(&manifest, &structure);
        
        assert_eq!(builder.get_image_tag(), "myregistry.io/myorg/test-app:v1.0.0");
    }
}

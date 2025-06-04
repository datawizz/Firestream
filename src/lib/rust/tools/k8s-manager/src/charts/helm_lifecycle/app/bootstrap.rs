//! Bootstrap script executor

use crate::core::{FirestreamError, Result};
use super::{AppManifest, schema::AppType};
use std::path::Path;
use std::collections::HashMap;
use tracing::{info, debug, warn};
use tokio::process::Command;

/// Bootstrap script executor
pub struct BootstrapExecutor {
    script_path: std::path::PathBuf,
}

impl BootstrapExecutor {
    /// Create new executor
    pub fn new(script_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            script_path: script_path.into(),
        }
    }
    
    /// Execute bootstrap script
    pub async fn execute(
        &self,
        working_dir: &Path,
        manifest: &AppManifest,
    ) -> Result<()> {
        info!("Executing bootstrap script: {:?}", self.script_path);
        
        // Check if script exists and is executable
        self.validate_script().await?;
        
        // Prepare environment variables
        let env_vars = self.prepare_environment(manifest);
        
        // Execute script
        let mut cmd = Command::new("/bin/bash");
        cmd.current_dir(working_dir);
        cmd.arg(&self.script_path);
        
        // Set environment variables
        for (key, value) in &env_vars {
            cmd.env(key, value);
        }
        
        debug!("Running bootstrap script with env vars: {:?}", env_vars.keys().collect::<Vec<_>>());
        
        let output = cmd.output().await
            .map_err(|e| FirestreamError::GeneralError(
                format!("Failed to execute bootstrap script: {}", e)
            ))?;
        
        // Log output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !stdout.is_empty() {
            for line in stdout.lines() {
                info!("[bootstrap] {}", line);
            }
        }
        
        if !stderr.is_empty() {
            for line in stderr.lines() {
                warn!("[bootstrap stderr] {}", line);
            }
        }
        
        // Check exit status
        if !output.status.success() {
            return Err(FirestreamError::GeneralError(
                format!("Bootstrap script failed with exit code: {:?}", output.status.code())
            ));
        }
        
        info!("Bootstrap script completed successfully");
        Ok(())
    }
    
    /// Validate bootstrap script
    async fn validate_script(&self) -> Result<()> {
        // Check if file exists
        if !self.script_path.exists() {
            return Err(FirestreamError::ConfigError(
                format!("Bootstrap script not found: {:?}", self.script_path)
            ));
        }
        
        // Check if file is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = tokio::fs::metadata(&self.script_path).await?;
            let permissions = metadata.permissions();
            
            // Check if any execute bit is set
            if permissions.mode() & 0o111 == 0 {
                // Make it executable
                info!("Making bootstrap script executable");
                let mut perms = metadata.permissions();
                perms.set_mode(perms.mode() | 0o755);
                tokio::fs::set_permissions(&self.script_path, perms).await?;
            }
        }
        
        Ok(())
    }
    
    /// Prepare environment variables for bootstrap script
    fn prepare_environment(&self, manifest: &AppManifest) -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        // App metadata
        env.insert("FIRESTREAM_APP_NAME".to_string(), manifest.app.name.clone());
        env.insert("FIRESTREAM_APP_VERSION".to_string(), manifest.app.version.clone());
        env.insert("FIRESTREAM_APP_TYPE".to_string(), format!("{:?}", manifest.app.app_type));
        
        // Build configuration
        let image_name = format!("{}/{}",
            manifest.build.registry.as_deref().unwrap_or("localhost:5000"),
            manifest.build.repository.as_deref().unwrap_or(&manifest.app.name)
        );
        env.insert("IMAGE_NAME".to_string(), image_name.clone());
        env.insert("IMAGE_TAG".to_string(), 
            manifest.build.tag.clone().unwrap_or_else(|| manifest.app.version.clone())
        );
        env.insert("FULL_IMAGE_NAME".to_string(), 
            format!("{}:{}", image_name, manifest.build.tag.as_deref().unwrap_or(&manifest.app.version))
        );
        
        // Local repository configuration
        env.insert("LOCAL_REPO".to_string(), 
            manifest.build.registry.clone().unwrap_or_else(|| "localhost:5000".to_string())
        );
        
        // Deployment configuration
        env.insert("NAMESPACE".to_string(), 
            manifest.deploy.namespace.clone().unwrap_or_else(|| manifest.app.name.clone())
        );
        
        // Add custom environment variables
        for (key, value) in &manifest.config.env {
            env.insert(format!("APP_{}", key), value.clone());
        }
        
        env
    }
    
    /// Execute bootstrap script with timeout
    pub async fn execute_with_timeout(
        &self,
        working_dir: &Path,
        manifest: &AppManifest,
        timeout_secs: u64,
    ) -> Result<()> {
        let timeout = tokio::time::Duration::from_secs(timeout_secs);
        
        match tokio::time::timeout(timeout, self.execute(working_dir, manifest)).await {
            Ok(result) => result,
            Err(_) => Err(FirestreamError::GeneralError(
                format!("Bootstrap script timed out after {} seconds", timeout_secs)
            )),
        }
    }
    
    /// Dry run - show what would be executed
    pub async fn dry_run(
        &self,
        working_dir: &Path,
        manifest: &AppManifest,
    ) -> Result<String> {
        self.validate_script().await?;
        
        let env_vars = self.prepare_environment(manifest);
        
        let mut output = String::new();
        output.push_str(&format!("Would execute: {:?}\n", self.script_path));
        output.push_str(&format!("Working directory: {:?}\n", working_dir));
        output.push_str("Environment variables:\n");
        
        for (key, value) in env_vars {
            output.push_str(&format!("  {}={}\n", key, value));
        }
        
        Ok(output)
    }
}

/// Bootstrap script generator for apps without one
pub struct BootstrapGenerator;

impl BootstrapGenerator {
    /// Generate a default bootstrap script
    pub async fn generate_default(
        app_dir: &Path,
        manifest: &AppManifest,
    ) -> Result<()> {
        let script_path = app_dir.join("bootstrap.sh");
        
        if script_path.exists() {
            warn!("Bootstrap script already exists, skipping generation");
            return Ok(());
        }
        
        info!("Generating default bootstrap script");
        
        let script_content = Self::generate_script_content(manifest);
        tokio::fs::write(&script_path, script_content).await?;
        
        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = tokio::fs::metadata(&script_path).await?;
            let mut perms = metadata.permissions();
            perms.set_mode(perms.mode() | 0o755);
            tokio::fs::set_permissions(&script_path, perms).await?;
        }
        
        info!("Generated bootstrap script at {:?}", script_path);
        Ok(())
    }
    
    /// Generate script content based on app type
    fn generate_script_content(manifest: &AppManifest) -> String {
        let mut script = String::from(r#"#!/bin/bash
# Firestream bootstrap script for {app_name}
# Generated automatically - customize as needed

set -e  # Exit on error

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
function info {
    echo -e "${GREEN}[INFO]${NC} $1"
}

function warn {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

function error {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Variables from Firestream
APP_NAME="${FIRESTREAM_APP_NAME}"
APP_VERSION="${FIRESTREAM_APP_VERSION}"
IMAGE_NAME="${IMAGE_NAME}"
IMAGE_TAG="${IMAGE_TAG}"
FULL_IMAGE_NAME="${FULL_IMAGE_NAME}"
NAMESPACE="${NAMESPACE}"

info "Bootstrapping $APP_NAME v$APP_VERSION"

"#);
        
        script = script.replace("{app_name}", &manifest.app.name);
        
        // Add app-type specific logic
        match manifest.app.app_type {
            AppType::Database => {
                script.push_str(r#"
# Database-specific setup
info "Setting up database prerequisites..."

# Example: Create persistent volume directories
# mkdir -p /data/${APP_NAME}

# Example: Initialize database schemas
# kubectl exec -n ${NAMESPACE} ${APP_NAME}-0 -- /bin/bash -c "initdb.sh"
"#);
            }
            AppType::MessageQueue => {
                script.push_str(r#"
# Message queue setup
info "Setting up message queue prerequisites..."

# Example: Create topics/exchanges
# kubectl exec -n ${NAMESPACE} ${APP_NAME}-0 -- /bin/bash -c "create-topics.sh"
"#);
            }
            AppType::Infrastructure => {
                script.push_str(r#"
# Infrastructure component setup
info "Setting up infrastructure prerequisites..."

# Example: Apply CRDs
# kubectl apply -f crds/

# Example: Create service accounts
# kubectl create serviceaccount ${APP_NAME} -n ${NAMESPACE} || true
"#);
            }
            _ => {
                // Default service setup
                script.push_str(r#"
# Service setup
info "Setting up service prerequisites..."

# Example: Create config maps
# kubectl create configmap ${APP_NAME}-config \
#   --from-file=config/ \
#   -n ${NAMESPACE} \
#   --dry-run=client -o yaml | kubectl apply -f -
"#);
            }
        }
        
        // Add dependency checks if any
        if !manifest.dependencies.is_empty() {
            script.push_str(r#"
# Check dependencies
info "Checking dependencies..."
"#);
            
            for (dep_name, _) in &manifest.dependencies {
                script.push_str(&format!(r#"
if ! kubectl get deployment {} -n {} &>/dev/null; then
    warn "Dependency '{}' not found - it should be deployed first"
fi
"#, dep_name, dep_name, dep_name));
            }
        }
        
        // Add Helm repository setup if using external chart
        if let Some(helm_config) = &manifest.deploy.helm {
            if let Some(repo) = &helm_config.repository {
                script.push_str(&format!(r#"
# Add Helm repository
info "Adding Helm repository..."
helm repo add {} {} || true
helm repo update
"#, manifest.app.name, repo));
            }
        }
        
        script.push_str(r#"
# Build and push Docker image (if Dockerfile exists)
if [ -f "Dockerfile" ]; then
    info "Building Docker image: $FULL_IMAGE_NAME"
    docker build -t "$FULL_IMAGE_NAME" . || error "Failed to build Docker image"
    
    info "Pushing Docker image to registry..."
    docker push "$FULL_IMAGE_NAME" || error "Failed to push Docker image"
    
    info "Docker image pushed successfully: $FULL_IMAGE_NAME"
else
    warn "No Dockerfile found, skipping image build"
fi

info "Bootstrap completed successfully!"
"#);
        
        script
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deploy::helm_lifecycle::app::schema::{AppMetadata, AppConfig};
    
    #[tokio::test]
    async fn test_environment_preparation() {
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
            config: AppConfig {
                env: {
                    let mut env = HashMap::new();
                    env.insert("KEY1".to_string(), "value1".to_string());
                    env
                },
                ..Default::default()
            },
            build: Default::default(),
            dependencies: Default::default(),
            environments: Default::default(),
            secrets: Default::default(),
            deploy: Default::default(),
        };
        
        let executor = BootstrapExecutor::new("/tmp/bootstrap.sh");
        let env = executor.prepare_environment(&manifest);
        
        assert_eq!(env.get("FIRESTREAM_APP_NAME"), Some(&"test-app".to_string()));
        assert_eq!(env.get("FIRESTREAM_APP_VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(env.get("APP_KEY1"), Some(&"value1".to_string()));
    }
}

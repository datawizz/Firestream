//! Firestream app manifest parser

use crate::core::{FirestreamError, Result};
use super::schema::*;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use async_trait::async_trait;

/// Manifest parser for firestream.toml files
pub struct ManifestParser;

impl ManifestParser {
    /// Parse a firestream.toml file
    pub async fn parse_file(path: impl AsRef<Path>) -> Result<AppManifest> {
        let path = path.as_ref();
        debug!("Parsing firestream.toml from {:?}", path);
        
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| FirestreamError::IoError(
                format!("Failed to read manifest file: {}", e)
            ))?;
        
        Self::parse_str(&content)
    }
    
    /// Parse firestream.toml from string
    pub fn parse_str(content: &str) -> Result<AppManifest> {
        toml::from_str(content)
            .map_err(|e| FirestreamError::ConfigError(
                format!("Failed to parse manifest: {}", e)
            ))
    }
    
    /// Validate manifest
    pub fn validate(manifest: &AppManifest) -> Result<()> {
        // Validate app name
        if manifest.app.name.is_empty() {
            return Err(FirestreamError::ConfigError(
                "App name cannot be empty".to_string()
            ));
        }
        
        // Validate app name format (DNS-1123 subdomain)
        if !Self::is_valid_app_name(&manifest.app.name) {
            return Err(FirestreamError::ConfigError(
                format!("Invalid app name '{}'. Must be a valid DNS subdomain", manifest.app.name)
            ));
        }
        
        // Validate version
        semver::Version::parse(&manifest.app.version)
            .map_err(|e| FirestreamError::ConfigError(
                format!("Invalid app version '{}': {}", manifest.app.version, e)
            ))?;
        
        // Validate dependencies
        for (dep_name, dep) in &manifest.dependencies {
            if !Self::is_valid_app_name(dep_name) {
                return Err(FirestreamError::ConfigError(
                    format!("Invalid dependency name '{}'", dep_name)
                ));
            }
            
            // Validate dependency version
            dep.version_req()?;
        }
        
        // Validate ports
        for port in &manifest.config.ports {
            if port.port == 0 {
                return Err(FirestreamError::ConfigError(
                    format!("Invalid port number: {} (port cannot be 0)", port.port)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check if name is valid DNS subdomain
    fn is_valid_app_name(name: &str) -> bool {
        // DNS-1123 subdomain: lowercase alphanumeric or '-', start/end with alphanumeric
        let re = regex::Regex::new(r"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$").unwrap();
        re.is_match(name) && name.len() <= 63
    }
    
    /// Resolve dependencies for an app
    pub async fn resolve_dependencies(
        manifest: &AppManifest,
        app_registry: &impl AppRegistry,
    ) -> Result<Vec<ResolvedDependency>> {
        let mut resolved = Vec::new();
        
        for (name, dep) in &manifest.dependencies {
            info!("Resolving dependency: {} {}", name, 
                match dep {
                    AppDependency::Version(v) => v,
                    AppDependency::Detailed { version, .. } => version,
                }
            );
            
            let app_info = app_registry.get_app(name).await?;
            let version_req = dep.version_req()?;
            
            // Find matching version
            let matching_version = app_info.versions.iter()
                .filter_map(|v| semver::Version::parse(v).ok())
                .find(|v| version_req.matches(v))
                .ok_or_else(|| FirestreamError::ConfigError(
                    format!("No matching version found for dependency {}", name)
                ))?;
            
            resolved.push(ResolvedDependency {
                name: name.clone(),
                version: matching_version.to_string(),
                manifest_path: app_info.manifest_path.clone(),
                should_wait: dep.should_wait(),
            });
        }
        
        Ok(resolved)
    }
    
    /// Load manifest with environment overrides
    pub async fn load_with_environment(
        path: impl AsRef<Path>,
        environment: Option<&str>,
    ) -> Result<AppManifest> {
        let mut manifest = Self::parse_file(path).await?;
        
        // Apply environment-specific overrides
        if let Some(env) = environment {
            if let Some(env_config) = manifest.environments.get(env).cloned() {
                info!("Applying environment overrides for '{}'", env);
                Self::apply_environment_overrides(&mut manifest, &env_config);
            }
        }
        
        Self::validate(&manifest)?;
        Ok(manifest)
    }
    
    /// Apply environment-specific overrides
    fn apply_environment_overrides(manifest: &mut AppManifest, env_config: &EnvironmentConfig) {
        // Merge environment variables
        manifest.config.env.extend(env_config.config.env.clone());
        
        // Override resources if specified
        if env_config.config.resources.cpu_request.is_some() {
            manifest.config.resources.cpu_request = env_config.config.resources.cpu_request.clone();
        }
        if env_config.config.resources.cpu_limit.is_some() {
            manifest.config.resources.cpu_limit = env_config.config.resources.cpu_limit.clone();
        }
        if env_config.config.resources.memory_request.is_some() {
            manifest.config.resources.memory_request = env_config.config.resources.memory_request.clone();
        }
        if env_config.config.resources.memory_limit.is_some() {
            manifest.config.resources.memory_limit = env_config.config.resources.memory_limit.clone();
        }
        
        // Override scaling if specified
        if env_config.config.scaling.replicas > 0 {
            manifest.config.scaling.replicas = env_config.config.scaling.replicas;
        }
        
        // Merge secrets
        manifest.secrets.inline.extend(env_config.secrets.inline.clone());
        manifest.secrets.refs.extend(env_config.secrets.refs.clone());
    }
}

/// Resolved dependency information
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub manifest_path: PathBuf,
    pub should_wait: bool,
}

/// App registry for dependency resolution
#[async_trait]
pub trait AppRegistry: Send + Sync {
    /// Get app information
    async fn get_app(&self, name: &str) -> Result<AppInfo>;
}

/// App information from registry
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub versions: Vec<String>,
    pub manifest_path: PathBuf,
}

/// Local file system app registry
pub struct LocalAppRegistry {
    /// Base directory containing all apps
    pub base_dir: PathBuf,
}

#[async_trait]
impl AppRegistry for LocalAppRegistry {
    async fn get_app(&self, name: &str) -> Result<AppInfo> {
        let app_dir = self.base_dir.join(name);
        
        if !app_dir.exists() {
            return Err(FirestreamError::ConfigError(
                format!("App '{}' not found in registry", name)
            ));
        }
        
        let manifest_path = app_dir.join("firestream.toml");
        let manifest = ManifestParser::parse_file(&manifest_path).await?;
        
        Ok(AppInfo {
            name: manifest.app.name,
            versions: vec![manifest.app.version],
            manifest_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_app_names() {
        assert!(ManifestParser::is_valid_app_name("my-app"));
        assert!(ManifestParser::is_valid_app_name("app123"));
        assert!(ManifestParser::is_valid_app_name("test-service-v2"));
        
        assert!(!ManifestParser::is_valid_app_name("My-App")); // uppercase
        assert!(!ManifestParser::is_valid_app_name("-app")); // starts with -
        assert!(!ManifestParser::is_valid_app_name("app-")); // ends with -
        assert!(!ManifestParser::is_valid_app_name("app_name")); // underscore
        assert!(!ManifestParser::is_valid_app_name("")); // empty
    }
    
    #[test]
    fn test_parse_manifest() {
        let toml = r#"
        [app]
        name = "test-service"
        version = "1.0.0"
        
        [dependencies]
        postgresql = "^13.0"
        
        [[config.ports]]
        name = "http"
        port = 8080
        "#;
        
        let manifest = ManifestParser::parse_str(toml).unwrap();
        assert_eq!(manifest.app.name, "test-service");
        
        let validation = ManifestParser::validate(&manifest);
        assert!(validation.is_ok());
    }
}

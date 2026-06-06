use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the helm/kubectl client wrappers in this crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to helm binary (if not in PATH)
    pub helm_path: Option<PathBuf>,
    
    /// Path to kubectl binary (if not in PATH)
    pub kubectl_path: Option<PathBuf>,
    
    /// Default namespace for deployments
    pub default_namespace: String,
    
    /// Default timeout for operations (in seconds)
    pub default_timeout: u64,
    
    /// Cache directory for extracted charts
    pub cache_dir: Option<PathBuf>,
    
    /// Enable debug logging for helm commands
    pub debug: bool,
    
    /// Kubeconfig path (if not using default)
    pub kubeconfig: Option<PathBuf>,
    
    /// Helm repository configuration
    pub repositories: Vec<Repository>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            helm_path: None,
            kubectl_path: None,
            default_namespace: "default".to_string(),
            default_timeout: 300, // 5 minutes
            cache_dir: None,
            debug: false,
            kubeconfig: None,
            repositories: vec![],
        }
    }
}

/// Helm repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository name
    pub name: String,
    
    /// Repository URL
    pub url: String,
    
    /// Username for authentication
    pub username: Option<String>,
    
    /// Password for authentication
    pub password: Option<String>,
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| crate::Error::InvalidConfig(e.to_string()))?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::Error::InvalidConfig(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get cache directory, creating it if necessary
    pub fn get_cache_dir(&self) -> crate::Result<PathBuf> {
        let cache_dir = if let Some(ref dir) = self.cache_dir {
            dir.clone()
        } else {
            directories::ProjectDirs::from("com", "firestream", "helm-manager")
                .ok_or_else(|| crate::Error::InvalidConfig("Failed to determine cache directory".to_string()))?
                .cache_dir()
                .to_path_buf()
        };

        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }

        Ok(cache_dir)
    }
}
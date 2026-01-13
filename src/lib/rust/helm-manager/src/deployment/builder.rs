use std::path::PathBuf;
use crate::{Result, Values};

/// Represents a deployment configuration
#[derive(Debug, Clone)]
pub struct Deployment {
    /// Chart name
    pub chart: String,
    
    /// Release name
    pub name: String,
    
    /// Target namespace
    pub namespace: String,
    
    /// Environment file path
    pub env_file: Option<PathBuf>,
    
    /// Environment variable prefix
    pub env_prefix: Option<String>,
    
    /// Values files (in order of precedence)
    pub values_files: Vec<PathBuf>,
    
    /// Inline values
    pub values: Option<Values>,
    
    /// Wait for deployment to be ready
    pub wait: bool,
    
    /// Atomic deployment (rollback on failure)
    pub atomic: bool,
    
    /// Timeout in seconds
    pub timeout: u64,
    
    /// Dependencies (other release names)
    pub dependencies: Vec<String>,
}

/// Builder for creating deployments
pub struct DeploymentBuilder {
    chart: String,
    name: Option<String>,
    namespace: String,
    env_file: Option<PathBuf>,
    env_prefix: Option<String>,
    values_files: Vec<PathBuf>,
    values: Option<Values>,
    wait: bool,
    atomic: bool,
    timeout: u64,
    dependencies: Vec<String>,
}

impl DeploymentBuilder {
    /// Create a new deployment builder
    pub fn new(chart: impl Into<String>) -> Self {
        Self {
            chart: chart.into(),
            name: None,
            namespace: "default".to_string(),
            env_file: None,
            env_prefix: None,
            values_files: Vec::new(),
            values: None,
            wait: true,
            atomic: false,
            timeout: 300,
            dependencies: Vec::new(),
        }
    }

    /// Set the release name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the namespace
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    /// Set the environment file
    pub fn env_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.env_file = Some(path.into());
        self
    }

    /// Set the environment variable prefix
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Add a values file
    pub fn values_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.values_files.push(path.into());
        self
    }

    /// Add multiple values files
    pub fn values_files(mut self, paths: Vec<PathBuf>) -> Self {
        self.values_files.extend(paths);
        self
    }

    /// Set inline values
    pub fn values(mut self, values: Values) -> Self {
        self.values = Some(values);
        self
    }

    /// Set a single value
    pub fn set_value(mut self, key: &str, value: serde_json::Value) -> Self {
        if self.values.is_none() {
            self.values = Some(Values::new());
        }
        if let Some(ref mut values) = self.values {
            values.set(key, value);
        }
        self
    }

    /// Wait for deployment to be ready
    pub fn wait(mut self) -> Self {
        self.wait = true;
        self
    }

    /// Don't wait for deployment
    pub fn no_wait(mut self) -> Self {
        self.wait = false;
        self
    }

    /// Enable atomic deployment
    pub fn atomic(mut self) -> Self {
        self.atomic = true;
        self
    }

    /// Set timeout in seconds
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout = seconds;
        self
    }

    /// Add a dependency
    pub fn depends_on(mut self, release: impl Into<String>) -> Self {
        self.dependencies.push(release.into());
        self
    }

    /// Add multiple dependencies
    pub fn dependencies(mut self, releases: Vec<String>) -> Self {
        self.dependencies.extend(releases);
        self
    }

    /// Build the deployment
    pub fn build(self) -> Result<Deployment> {
        let name = self.name.unwrap_or_else(|| self.chart.clone());

        Ok(Deployment {
            chart: self.chart,
            name,
            namespace: self.namespace,
            env_file: self.env_file,
            env_prefix: self.env_prefix,
            values_files: self.values_files,
            values: self.values,
            wait: self.wait,
            atomic: self.atomic,
            timeout: self.timeout,
            dependencies: self.dependencies,
        })
    }
}

impl Deployment {
    /// Create a builder for the given chart
    pub fn builder(chart: impl Into<String>) -> DeploymentBuilder {
        DeploymentBuilder::new(chart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_builder() {
        let deployment = Deployment::builder("postgresql")
            .name("my-db")
            .namespace("production")
            .env_file("/etc/config/.env")
            .atomic()
            .timeout(600)
            .build()
            .unwrap();

        assert_eq!(deployment.chart, "postgresql");
        assert_eq!(deployment.name, "my-db");
        assert_eq!(deployment.namespace, "production");
        assert_eq!(deployment.env_file.unwrap().to_str().unwrap(), "/etc/config/.env");
        assert!(deployment.atomic);
        assert_eq!(deployment.timeout, 600);
    }

    #[test]
    fn test_deployment_with_values() {
        let mut values = Values::new();
        values.set("postgresql.auth.password", serde_json::json!("secret"));

        let deployment = Deployment::builder("postgresql")
            .values(values)
            .build()
            .unwrap();

        assert!(deployment.values.is_some());
    }
}
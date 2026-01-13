use std::path::PathBuf;
use crate::{Deployment, DeploymentBuilder, Result};

/// Represents a stack of related deployments
#[derive(Debug, Clone)]
pub struct Stack {
    /// Stack name
    pub name: String,
    
    /// Default namespace for all deployments
    pub namespace: String,
    
    /// Default environment file
    pub env_file: Option<PathBuf>,
    
    /// Deployments in the stack
    pub deployments: Vec<Deployment>,
}

/// Builder for creating stacks
pub struct StackBuilder {
    name: Option<String>,
    namespace: String,
    env_file: Option<PathBuf>,
    deployments: Vec<Deployment>,
}

impl StackBuilder {
    /// Create a new stack builder
    pub fn new() -> Self {
        Self {
            name: None,
            namespace: "default".to_string(),
            env_file: None,
            deployments: Vec::new(),
        }
    }

    /// Set the stack name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the default namespace
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    /// Set the default environment file
    pub fn env_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.env_file = Some(path.into());
        self
    }

    /// Add a deployment to the stack
    pub fn add<F>(mut self, chart: &str, configure: F) -> Self
    where
        F: FnOnce(DeploymentBuilder) -> DeploymentBuilder,
    {
        let mut builder = Deployment::builder(chart)
            .namespace(&self.namespace);

        // Apply default env file if set
        if let Some(ref env_file) = self.env_file {
            builder = builder.env_file(env_file);
        }

        // Apply user configuration
        builder = configure(builder);

        if let Ok(deployment) = builder.build() {
            self.deployments.push(deployment);
        }

        self
    }

    /// Add a pre-built deployment
    pub fn add_deployment(mut self, deployment: Deployment) -> Self {
        self.deployments.push(deployment);
        self
    }

    /// Build the stack
    pub fn build(self) -> Result<Stack> {
        let name = self.name.unwrap_or_else(|| "stack".to_string());

        Ok(Stack {
            name,
            namespace: self.namespace,
            env_file: self.env_file,
            deployments: self.deployments,
        })
    }
}

impl Default for StackBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    /// Create a new stack builder
    pub fn builder() -> StackBuilder {
        StackBuilder::new()
    }

    /// Get deployments in dependency order
    pub fn get_ordered_deployments(&self) -> Vec<&Deployment> {
        // TODO: Implement topological sort based on dependencies
        // For now, return in the order they were added
        self.deployments.iter().collect()
    }

    /// Find a deployment by name
    pub fn find_deployment(&self, name: &str) -> Option<&Deployment> {
        self.deployments.iter().find(|d| d.name == name)
    }

    /// Check if all dependencies are satisfied
    pub fn validate_dependencies(&self) -> Result<()> {
        for deployment in &self.deployments {
            for dep in &deployment.dependencies {
                if !self.deployments.iter().any(|d| d.name == *dep) {
                    return Err(crate::Error::InvalidConfig(format!(
                        "Deployment '{}' depends on '{}' which is not in the stack",
                        deployment.name, dep
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_builder() {
        let stack = Stack::builder()
            .name("my-stack")
            .namespace("production")
            .env_file("/etc/config/.env")
            .add("postgresql", |d| d.name("database"))
            .add("redis", |d| d.name("cache"))
            .add("kafka", |d| d
                .name("streaming")
                .depends_on("database"))
            .build()
            .unwrap();

        assert_eq!(stack.name, "my-stack");
        assert_eq!(stack.namespace, "production");
        assert_eq!(stack.deployments.len(), 3);
        
        // Check dependency
        let kafka = stack.find_deployment("streaming").unwrap();
        assert!(kafka.dependencies.contains(&"database".to_string()));
    }

    #[test]
    fn test_stack_validation() {
        let stack = Stack::builder()
            .add("postgresql", |d| d.name("database"))
            .add("app", |d| d
                .name("myapp")
                .depends_on("database"))
            .build()
            .unwrap();

        assert!(stack.validate_dependencies().is_ok());

        // Test invalid dependency
        let invalid_stack = Stack::builder()
            .add("app", |d| d
                .name("myapp")
                .depends_on("nonexistent"))
            .build()
            .unwrap();

        assert!(invalid_stack.validate_dependencies().is_err());
    }
}
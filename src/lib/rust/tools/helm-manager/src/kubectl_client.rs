use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use crate::{Error, Result};

/// Kubectl CLI wrapper
pub struct KubectlClient {
    /// Path to kubectl binary
    kubectl_path: PathBuf,
    /// Kubeconfig path
    kubeconfig: Option<PathBuf>,
}

impl KubectlClient {
    /// Create a new KubectlClient
    pub fn new() -> Result<Self> {
        let kubectl_path = which::which("kubectl")
            .map_err(|_| Error::KubectlNotFound)?;

        Ok(Self {
            kubectl_path,
            kubeconfig: None,
        })
    }

    /// Create a new KubectlClient with custom configuration
    pub fn with_config(kubectl_path: Option<PathBuf>, kubeconfig: Option<PathBuf>) -> Result<Self> {
        let kubectl_path = if let Some(path) = kubectl_path {
            path
        } else {
            which::which("kubectl").map_err(|_| Error::KubectlNotFound)?
        };

        Ok(Self {
            kubectl_path,
            kubeconfig,
        })
    }

    /// Check if a namespace exists
    pub async fn namespace_exists(&self, name: &str) -> Result<bool> {
        let mut cmd = self.base_command();
        cmd.arg("get")
            .arg("namespace")
            .arg(name);

        let output = cmd.output().await?;
        Ok(output.status.success())
    }

    /// Create a namespace
    pub async fn create_namespace(&self, name: &str) -> Result<()> {
        // Check if namespace already exists
        if self.namespace_exists(name).await? {
            return Ok(());
        }

        let mut cmd = self.base_command();
        cmd.arg("create")
            .arg("namespace")
            .arg(name);

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::KubectlCommandFailed(format!(
                "Failed to create namespace: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Get pods for a release
    pub async fn get_pods(&self, namespace: &str, selector: &str) -> Result<Vec<String>> {
        let mut cmd = self.base_command();
        cmd.arg("get")
            .arg("pods")
            .arg("-n").arg(namespace)
            .arg("-l").arg(selector)
            .arg("-o").arg("jsonpath={.items[*].metadata.name}");

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::KubectlCommandFailed(format!(
                "Failed to get pods: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let pods: Vec<String> = stdout
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Ok(pods)
    }

    /// Wait for pods to be ready
    pub async fn wait_for_ready(&self, namespace: &str, selector: &str, timeout: u64) -> Result<()> {
        let mut cmd = self.base_command();
        cmd.arg("wait")
            .arg("--for=condition=ready")
            .arg("pod")
            .arg("-n").arg(namespace)
            .arg("-l").arg(selector)
            .arg(format!("--timeout={}s", timeout));

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::KubectlCommandFailed(format!(
                "Pods failed to become ready: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Get resource YAML
    pub async fn get_resource_yaml(&self, kind: &str, name: &str, namespace: &str) -> Result<String> {
        let mut cmd = self.base_command();
        cmd.arg("get")
            .arg(kind)
            .arg(name)
            .arg("-n").arg(namespace)
            .arg("-o").arg("yaml");

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::KubectlCommandFailed(format!(
                "Failed to get resource: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Apply a resource from YAML
    pub async fn apply_yaml(&self, yaml: &str, namespace: Option<&str>) -> Result<()> {
        let mut cmd = self.base_command();
        cmd.arg("apply")
            .arg("-f")
            .arg("-");
        
        if let Some(ns) = namespace {
            cmd.arg("-n").arg(ns);
        }

        cmd.stdin(Stdio::piped());

        let mut child = cmd.spawn()?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(yaml.as_bytes()).await?;
        }

        let output = child.wait_with_output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::KubectlCommandFailed(format!(
                "Failed to apply resource: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Delete a resource
    pub async fn delete_resource(&self, kind: &str, name: &str, namespace: &str) -> Result<()> {
        let mut cmd = self.base_command();
        cmd.arg("delete")
            .arg(kind)
            .arg(name)
            .arg("-n").arg(namespace);

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore if resource not found
            if !stderr.contains("NotFound") {
                return Err(Error::KubectlCommandFailed(format!(
                    "Failed to delete resource: {}",
                    stderr
                )));
            }
        }

        Ok(())
    }

    /// Create base command with common arguments
    fn base_command(&self) -> Command {
        let mut cmd = Command::new(&self.kubectl_path);
        
        if let Some(ref kubeconfig) = self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        cmd.kill_on_drop(true);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kubectl_client_creation() {
        // This will fail if kubectl is not installed
        let result = KubectlClient::new();
        if which::which("kubectl").is_ok() {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
}
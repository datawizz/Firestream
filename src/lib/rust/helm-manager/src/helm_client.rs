use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use std::process::Stdio;
use tempfile::NamedTempFile;
use tokio::process::Command;

use crate::{Error, Release, ReleaseStatus, Result};

/// Helm CLI wrapper
pub struct HelmClient {
    /// Path to helm binary
    helm_path: PathBuf,
    /// Kubeconfig path
    kubeconfig: Option<PathBuf>,
    /// Enable debug mode
    debug: bool,
}

impl HelmClient {
    /// Create a new HelmClient
    pub fn new() -> Result<Self> {
        let helm_path = which::which("helm")
            .map_err(|_| Error::HelmNotFound)?;

        Ok(Self {
            helm_path,
            kubeconfig: None,
            debug: false,
        })
    }

    /// Create a new HelmClient with custom configuration
    pub fn with_config(helm_path: Option<PathBuf>, kubeconfig: Option<PathBuf>, debug: bool) -> Result<Self> {
        let helm_path = if let Some(path) = helm_path {
            path
        } else {
            which::which("helm").map_err(|_| Error::HelmNotFound)?
        };

        Ok(Self {
            helm_path,
            kubeconfig,
            debug,
        })
    }

    /// Install a new release
    pub async fn install(
        &self,
        name: &str,
        chart: &str,
        namespace: &str,
        values: JsonValue,
        wait: bool,
        atomic: bool,
    ) -> Result<Release> {
        let values_file = self.write_values_file(&values).await?;
        
        let mut cmd = self.base_command();
        cmd.arg("install")
            .arg(name)
            .arg(chart)
            .arg("--namespace").arg(namespace)
            .arg("--values").arg(values_file.path())
            .arg("--output").arg("json");

        if wait {
            cmd.arg("--wait");
        }
        if atomic {
            cmd.arg("--atomic");
        }

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::HelmCommandFailed(format!(
                "helm install failed: {}",
                stderr
            )));
        }

        // Parse the output
        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_release_output(&stdout)
    }

    /// Upgrade an existing release
    pub async fn upgrade(
        &self,
        name: &str,
        chart: &str,
        namespace: &str,
        values: JsonValue,
        wait: bool,
        atomic: bool,
    ) -> Result<Release> {
        let values_file = self.write_values_file(&values).await?;
        
        let mut cmd = self.base_command();
        cmd.arg("upgrade")
            .arg(name)
            .arg(chart)
            .arg("--namespace").arg(namespace)
            .arg("--values").arg(values_file.path())
            .arg("--output").arg("json");

        if wait {
            cmd.arg("--wait");
        }
        if atomic {
            cmd.arg("--atomic");
        }

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::HelmCommandFailed(format!(
                "helm upgrade failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_release_output(&stdout)
    }

    /// Uninstall a release
    pub async fn uninstall(&self, name: &str) -> Result<()> {
        let mut cmd = self.base_command();
        cmd.arg("uninstall").arg(name);

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::HelmCommandFailed(format!(
                "helm uninstall failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Rollback a release
    pub async fn rollback(&self, name: &str, revision: u32) -> Result<()> {
        let mut cmd = self.base_command();
        cmd.arg("rollback")
            .arg(name)
            .arg(revision.to_string());

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::RollbackFailed(format!(
                "helm rollback failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Get release status
    pub async fn status(&self, name: &str) -> Result<ReleaseStatus> {
        let mut cmd = self.base_command();
        cmd.arg("status")
            .arg(name)
            .arg("--output").arg("json");

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") {
                return Err(Error::ReleaseNotFound(name.to_string()));
            }
            return Err(Error::HelmCommandFailed(format!(
                "helm status failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: JsonValue = serde_json::from_str(&stdout)
            .map_err(|e| Error::Serialization(format!("Failed to parse helm status output: {}", e)))?;

        let status_str = json["info"]["status"]
            .as_str()
            .unwrap_or("unknown");

        status_str.parse()
    }

    /// List releases
    pub async fn list(&self) -> Result<Vec<Release>> {
        let mut cmd = self.base_command();
        cmd.arg("list")
            .arg("--all-namespaces")
            .arg("--output").arg("json");

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::HelmCommandFailed(format!(
                "helm list failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let releases: Vec<JsonValue> = serde_json::from_str(&stdout)
            .map_err(|e| Error::Serialization(format!("Failed to parse helm list output: {}", e)))?;

        releases.into_iter()
            .map(|r| self.parse_release_json(&r))
            .collect()
    }

    /// Create base command with common arguments
    fn base_command(&self) -> Command {
        let mut cmd = Command::new(&self.helm_path);
        
        if let Some(ref kubeconfig) = self.kubeconfig {
            cmd.env("KUBECONFIG", kubeconfig);
        }
        
        if self.debug {
            cmd.arg("--debug");
        }
        
        cmd.kill_on_drop(true);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        cmd
    }

    /// Write values to a temporary file
    async fn write_values_file(&self, values: &JsonValue) -> Result<NamedTempFile> {
        let yaml_content = serde_yaml::to_string(values)?;
        let mut temp_file = NamedTempFile::new()?;
        
        use std::io::Write;
        temp_file.write_all(yaml_content.as_bytes())?;
        temp_file.flush()?;
        
        Ok(temp_file)
    }

    /// Parse release output from helm install/upgrade
    fn parse_release_output(&self, output: &str) -> Result<Release> {
        let json: JsonValue = serde_json::from_str(output)
            .map_err(|e| Error::Serialization(format!("Failed to parse helm output: {}", e)))?;
        
        self.parse_release_json(&json)
    }

    /// Parse release from JSON
    fn parse_release_json(&self, json: &JsonValue) -> Result<Release> {
        let name = json["name"]
            .as_str()
            .ok_or_else(|| Error::Serialization("Missing release name".to_string()))?
            .to_string();

        let namespace = json["namespace"]
            .as_str()
            .unwrap_or("default")
            .to_string();

        let chart = json["chart"]
            .as_str()
            .ok_or_else(|| Error::Serialization("Missing chart name".to_string()))?
            .to_string();

        let chart_version = json["chart_version"]
            .as_str()
            .or_else(|| json["version"].as_str())
            .unwrap_or("unknown")
            .to_string();

        let app_version = json["app_version"]
            .as_str()
            .map(|s| s.to_string());

        let revision = json["revision"]
            .as_u64()
            .unwrap_or(1) as u32;

        let status_str = json["status"]
            .as_str()
            .or_else(|| json["info"]["status"].as_str())
            .unwrap_or("unknown");

        let status: ReleaseStatus = status_str.parse()?;

        let updated = if let Some(updated_str) = json["updated"].as_str() {
            DateTime::parse_from_rfc3339(updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now())
        } else {
            Utc::now()
        };

        let notes = json["info"]["notes"]
            .as_str()
            .map(|s| s.to_string());

        Ok(Release {
            name,
            namespace,
            chart,
            chart_version,
            app_version,
            revision,
            status,
            updated,
            notes,
            values: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_release_status() {
        assert_eq!("deployed".parse::<ReleaseStatus>().unwrap(), ReleaseStatus::Deployed);
        assert_eq!("failed".parse::<ReleaseStatus>().unwrap(), ReleaseStatus::Failed);
        assert_eq!("pending-install".parse::<ReleaseStatus>().unwrap(), ReleaseStatus::PendingInstall);
    }
}
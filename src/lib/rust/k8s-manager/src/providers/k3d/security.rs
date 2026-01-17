//! Security operations for K3D clusters

use crate::{K8sManagerError, Result, CertificateConfig};
use crate::providers::k3d::manager::K3dClusterManager;
use tokio::process::Command;
use tokio::fs;
use tracing::{info, debug};
use std::collections::HashMap;
use tempfile;

impl K3dClusterManager {
    /// Generate TLS certificate
    pub(crate) async fn generate_tls_certificate(&self, config: &CertificateConfig) -> Result<()> {
        let cert_dir = dirs::home_dir()
            .ok_or_else(|| K8sManagerError::GeneralError("Cannot find home directory".into()))?
            .join("certs");
        
        fs::create_dir_all(&cert_dir).await?;
        
        let subject = format!(
            "/emailAddress={}/C={}/ST={}/L={}/O={}/OU={}/CN={}",
            config.email,
            config.country,
            config.state,
            config.locality,
            config.organization,
            config.organizational_unit,
            config.common_name
        );
        
        let output = Command::new("openssl")
            .args(&[
                "req",
                "-new",
                "-newkey", "rsa:4096",
                "-days", "365",
                "-nodes",
                "-x509",
                "-subj", &subject,
                "-keyout", cert_dir.join("server.key").to_str().unwrap(),
                "-out", cert_dir.join("server.crt").to_str().unwrap(),
            ])
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(K8sManagerError::TlsError(
                format!("Failed to generate certificate: {}", 
                    String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        debug!("Generated TLS certificate");
        Ok(())
    }
    
    /// Configure TLS certificates
    pub async fn configure_tls(&self) -> Result<()> {
        // Check if secret already exists
        let output = Command::new("kubectl")
            .args(&["get", "secret", &self.config.tls.secret_name])
            .output()
            .await?;
        
        if output.status.success() {
            info!("TLS secret '{}' already exists", self.config.tls.secret_name);
            return Ok(());
        }
        
        info!("Configuring TLS certificates");
        
        // Generate certificate
        self.generate_tls_certificate(&self.config.tls.certificate_config).await?;
        
        // Create secret
        let cert_dir = dirs::home_dir()
            .ok_or_else(|| K8sManagerError::GeneralError("Cannot find home directory".into()))?
            .join("certs");
        
        let status = Command::new("kubectl")
            .args(&[
                "create",
                "secret",
                "tls",
                &self.config.tls.secret_name,
                &format!("--cert={}", cert_dir.join("server.crt").display()),
                &format!("--key={}", cert_dir.join("server.key").display()),
            ])
            .status()
            .await?;
        
        if !status.success() {
            return Err(K8sManagerError::TlsError("Failed to create TLS secret".to_string()));
        }
        
        info!("TLS secret '{}' created", self.config.tls.secret_name);
        Ok(())
    }
    
    /// Create a generic secret
    pub async fn create_secret(&self, name: &str, namespace: &str, data: HashMap<String, Vec<u8>>) -> Result<()> {
        // Create a temporary directory for secret files
        let temp_dir = tempfile::tempdir()?;
        
        // Write each secret data to a file
        let mut file_args = Vec::new();
        for (key, value) in data {
            let file_path = temp_dir.path().join(&key);
            fs::write(&file_path, value).await?;
            file_args.push(format!("--from-file={}={}", key, file_path.display()));
        }
        
        let mut cmd = Command::new("kubectl");
        cmd.args(&["create", "secret", "generic", name, "-n", namespace]);
        cmd.args(&file_args);
        
        let status = cmd.status().await?;
        
        if !status.success() {
            return Err(K8sManagerError::GeneralError("Failed to create secret".to_string()));
        }
        
        Ok(())
    }
    
    /// Get a secret's data
    pub async fn get_secret(&self, name: &str, namespace: &str) -> Result<HashMap<String, Vec<u8>>> {
        let output = Command::new("kubectl")
            .args(&[
                "get", "secret", name,
                "-n", namespace,
                "-o", "json"
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(K8sManagerError::GeneralError("Failed to get secret".to_string()));
        }
        
        let secret_json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut result = HashMap::new();
        
        if let Some(data) = secret_json["data"].as_object() {
            for (key, value) in data {
                if let Some(encoded) = value.as_str() {
                    // Decode base64
                    use base64::{Engine as _, engine::general_purpose};
                    if let Ok(decoded) = general_purpose::STANDARD.decode(encoded) {
                        result.insert(key.clone(), decoded);
                    }
                }
            }
        }
        
        Ok(result)
    }
}
//! Advanced K3D cluster management
//!
//! This module provides comprehensive K3D cluster management with state integration,
//! matching the functionality of the shell scripts with additional features.

use crate::core::{FirestreamError, Result};
use crate::state::schema::{K3dClusterConfig, K3dDevModeConfig, CertificateConfig};
use std::path::PathBuf;
use tokio::process::Command;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, debug, warn};
use std::collections::HashMap;

/// K3D cluster manager with advanced features
pub struct K3dClusterManager {
    config: K3dClusterConfig,
    project_root: PathBuf,
}

impl K3dClusterManager {
    /// Create a new K3D cluster manager
    pub fn new(config: K3dClusterConfig) -> Self {
        Self {
            config,
            project_root: std::env::current_dir().unwrap_or_default(),
        }
    }
    
    /// Full cluster setup matching bootstrap.sh functionality
    pub async fn setup_cluster(&self) -> Result<()> {
        info!("Setting up K3D cluster '{}'", self.config.name);
        
        // Ensure .kube directory exists
        self.ensure_kube_directory().await?;
        
        // Create registry if enabled
        if self.config.registry.enabled {
            self.setup_registry().await?;
        }
        
        // Create or reconnect cluster
        if self.cluster_exists().await? {
            info!("Cluster '{}' already exists, reconnecting...", self.config.name);
            self.reconnect_cluster().await?;
        } else {
            info!("Creating new cluster '{}'", self.config.name);
            self.create_cluster().await?;
            self.wait_for_cluster().await?;
        }
        
        // Configure TLS if enabled
        if self.config.tls.enabled {
            self.configure_tls().await?;
        }
        
        // Configure network
        if self.config.network.configure_routes {
            self.setup_ip_routes().await?;
        }
        
        if self.config.network.configure_dns {
            self.configure_dns().await?;
        }
        
        if self.config.network.patch_etc_hosts {
            self.patch_etc_hosts().await?;
        }
        
        // Test DNS resolution
        self.test_dns_resolution().await?;
        
        // Setup dev mode if enabled
        if let Some(dev_config) = &self.config.dev_mode {
            if dev_config.port_forward_all {
                self.setup_port_forwarding(dev_config).await?;
            }
        }
        
        info!("K3D cluster '{}' setup complete", self.config.name);
        Ok(())
    }
    
    /// Ensure .kube directory exists
    async fn ensure_kube_directory(&self) -> Result<()> {
        let kube_dir = dirs::home_dir()
            .ok_or_else(|| FirestreamError::GeneralError("Cannot find home directory".into()))?
            .join(".kube");
        
        fs::create_dir_all(&kube_dir).await
            .map_err(|e| FirestreamError::IoError(format!("Failed to create .kube directory: {}", e)))?;
        
        Ok(())
    }
    
    /// Check if cluster exists
    async fn cluster_exists(&self) -> Result<bool> {
        let output = Command::new("k3d")
            .args(&["cluster", "list"])
            .output()
            .await
            .map_err(|e| FirestreamError::GeneralError(format!("Failed to list clusters: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(&self.config.name))
    }
    
    /// Setup container registry
    async fn setup_registry(&self) -> Result<()> {
        let registry_name = format!("{}.{}", self.config.name, self.config.registry.name);
        
        // Check if registry exists
        let output = Command::new("k3d")
            .args(&["registry", "list"])
            .output()
            .await?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(&registry_name) {
            info!("Registry '{}' already exists", registry_name);
            return Ok(());
        }
        
        info!("Creating registry '{}'", registry_name);
        
        let status = Command::new("k3d")
            .args(&[
                "registry",
                "create",
                &registry_name,
                "--port",
                &self.config.registry.port.to_string(),
                "--cluster",
                &self.config.name,
            ])
            .status()
            .await
            .map_err(|e| FirestreamError::GeneralError(format!("Failed to create registry: {}", e)))?;
        
        if !status.success() {
            return Err(FirestreamError::GeneralError("Failed to create registry".to_string()));
        }
        
        Ok(())
    }
    
    /// Reconnect to existing cluster
    async fn reconnect_cluster(&self) -> Result<()> {
        info!("Reconnecting to cluster '{}'", self.config.name);
        
        // Write kubeconfig
        let status = Command::new("k3d")
            .args(&[
                "kubeconfig",
                "write",
                &self.config.name,
                "--kubeconfig-switch-context",
            ])
            .status()
            .await?;
        
        if !status.success() {
            return Err(FirestreamError::GeneralError("Failed to write kubeconfig".to_string()));
        }
        
        // Move kubeconfig to standard location
        let k3d_config = dirs::home_dir()
            .ok_or_else(|| FirestreamError::GeneralError("Cannot find home directory".into()))?
            .join(".config")
            .join("k3d")
            .join(format!("kubeconfig-{}.yaml", self.config.name));
        
        let kube_config = dirs::home_dir()
            .ok_or_else(|| FirestreamError::GeneralError("Cannot find home directory".into()))?
            .join(".kube")
            .join("config");
        
        if k3d_config.exists() {
            fs::rename(&k3d_config, &kube_config).await
                .map_err(|e| FirestreamError::IoError(format!("Failed to move kubeconfig: {}", e)))?;
        }
        
        // Set context
        let status = Command::new("kubectl")
            .args(&[
                "config",
                "--kubeconfig",
                kube_config.to_str().unwrap(),
                "use-context",
                &format!("k3d-{}", self.config.name),
            ])
            .status()
            .await?;
        
        if !status.success() {
            warn!("Failed to set kubectl context");
        }
        
        // Test kubectl connection
        self.test_kubectl().await?;
        
        Ok(())
    }
    
    /// Create new cluster
    async fn create_cluster(&self) -> Result<()> {
        let registry_name = format!("{}.{}", self.config.name, self.config.registry.name);
        
        let mut cmd = Command::new("k3d");
        cmd.args(&[
            "cluster",
            "create",
            &self.config.name,
            "--api-port",
            &self.config.api_port.to_string(),
            "-p",
            &format!("{}:{}@loadbalancer", self.config.http_port, self.config.http_port),
            "-p",
            &format!("{}:{}@loadbalancer", self.config.https_port, self.config.https_port),
            "--registry-use",
            &format!("k3d-{}:{}", registry_name, self.config.registry.port),
            "--image",
            &format!("rancher/k3s:{}", self.config.k3s_version),
            "--servers",
            &self.config.servers.to_string(),
            "--agents",
            &self.config.agents.to_string(),
        ]);
        
        let status = cmd.status().await
            .map_err(|e| FirestreamError::GeneralError(format!("Failed to create cluster: {}", e)))?;
        
        if !status.success() {
            return Err(FirestreamError::GeneralError("Failed to create cluster".to_string()));
        }
        
        // Set kubectl context
        let _ = Command::new("kubectl")
            .args(&[
                "config",
                "use-context",
                &format!("k3d-{}", self.config.name),
            ])
            .status()
            .await;
        
        Ok(())
    }
    
    /// Wait for cluster to be ready
    async fn wait_for_cluster(&self) -> Result<()> {
        info!("Waiting for CoreDNS to be ready...");
        
        let status = Command::new("kubectl")
            .args(&[
                "wait",
                "--namespace=kube-system",
                "--for=condition=available",
                "--timeout=300s",
                "--all",
                "deployments",
            ])
            .status()
            .await?;
        
        if !status.success() {
            return Err(FirestreamError::GeneralError("Timeout waiting for cluster".to_string()));
        }
        
        Ok(())
    }
    
    /// Test kubectl connection
    async fn test_kubectl(&self) -> Result<()> {
        let output = Command::new("kubectl")
            .args(&["version", "--output=json"])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(FirestreamError::GeneralError("kubectl connection failed".to_string()));
        }
        
        let version_json = String::from_utf8_lossy(&output.stdout);
        let version: serde_json::Value = serde_json::from_str(&version_json)
            .map_err(|e| FirestreamError::GeneralError(format!("Failed to parse kubectl version: {}", e)))?;
        
        if version["clientVersion"]["gitVersion"].is_null() || 
           version["serverVersion"]["gitVersion"].is_null() {
            return Err(FirestreamError::GeneralError("kubectl version check failed".to_string()));
        }
        
        info!("kubectl connection verified");
        Ok(())
    }
    
    /// Configure TLS certificates
    async fn configure_tls(&self) -> Result<()> {
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
            .ok_or_else(|| FirestreamError::GeneralError("Cannot find home directory".into()))?
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
            return Err(FirestreamError::GeneralError("Failed to create TLS secret".to_string()));
        }
        
        info!("TLS secret '{}' created", self.config.tls.secret_name);
        Ok(())
    }
    
    /// Generate TLS certificate
    async fn generate_tls_certificate(&self, config: &CertificateConfig) -> Result<()> {
        let cert_dir = dirs::home_dir()
            .ok_or_else(|| FirestreamError::GeneralError("Cannot find home directory".into()))?
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
            return Err(FirestreamError::GeneralError(
                format!("Failed to generate certificate: {}", 
                    String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        debug!("Generated TLS certificate");
        Ok(())
    }
    
    /// Setup IP routes for pod and service CIDRs
    async fn setup_ip_routes(&self) -> Result<()> {
        info!("Setting up IP routes");
        
        // Get k3d server IP
        let output = Command::new("docker")
            .args(&[
                "container",
                "inspect",
                &format!("k3d-{}-server-0", self.config.name),
                "--format",
                "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}",
            ])
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(FirestreamError::GeneralError("Failed to get server IP".to_string()));
        }
        
        let server_ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!("K3D server IP: {}", server_ip);
        
        // Check and add routes
        for cidr in [&self.config.network.pod_cidr, &self.config.network.service_cidr] {
            // Check if route exists
            let check = Command::new("sudo")
                .args(&["ip", "route", "show", cidr])
                .output()
                .await?;
            
            if check.stdout.is_empty() {
                // Add route
                let status = Command::new("sudo")
                    .args(&["ip", "route", "add", cidr, "via", &server_ip])
                    .status()
                    .await?;
                
                if status.success() {
                    info!("Added route for {}", cidr);
                } else {
                    warn!("Failed to add route for {}", cidr);
                }
            } else {
                debug!("Route for {} already exists", cidr);
            }
        }
        
        Ok(())
    }
    
    /// Configure DNS to use Kubernetes DNS
    async fn configure_dns(&self) -> Result<()> {
        info!("Configuring DNS");
        
        // Get Kubernetes DNS IP
        let output = Command::new("kubectl")
            .args(&[
                "get", "svc",
                "-n", "kube-system",
                "kube-dns",
                "-o", "jsonpath={.spec.clusterIP}",
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(FirestreamError::GeneralError("Failed to get Kubernetes DNS IP".to_string()));
        }
        
        let dns_ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!("Kubernetes DNS IP: {}", dns_ip);
        
        // Update resolv.conf
        let resolv_content = format!(
            "search svc.cluster.local cluster.local\nnameserver {}\noptions edns0 trust-ad",
            dns_ip
        );
        
        let mut child = Command::new("sudo")
            .args(&["tee", "/etc/resolv.conf"])
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(resolv_content.as_bytes()).await?;
        }
        
        let status = child.wait().await?;
        
        if !status.success() {
            warn!("Failed to update resolv.conf");
        }
        
        Ok(())
    }
    
    /// Patch /etc/hosts with hostname
    async fn patch_etc_hosts(&self) -> Result<()> {
        let hostname_output = Command::new("hostname")
            .output()
            .await?;
        
        let hostname = String::from_utf8_lossy(&hostname_output.stdout).trim().to_string();
        
        // Check if hostname already in /etc/hosts
        let check = Command::new("grep")
            .args(&["-q", &hostname, "/etc/hosts"])
            .status()
            .await?;
        
        if !check.success() {
            // Add hostname
            let entry = format!("127.0.0.1 {}", hostname);
            let mut child = Command::new("sudo")
                .args(&["tee", "-a", "/etc/hosts"])
                .stdin(std::process::Stdio::piped())
                .spawn()?;
            
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(format!("\n{}\n", entry).as_bytes()).await?;
            }
            
            let status = child.wait().await?;
            
            if status.success() {
                info!("Added hostname {} to /etc/hosts", hostname);
            }
        }
        
        Ok(())
    }
    
    /// Test DNS resolution
    async fn test_dns_resolution(&self) -> Result<()> {
        debug!("Testing DNS resolution");
        
        let max_attempts = 4;
        let mut wait_time = 1;
        
        for attempt in 0..max_attempts {
            let output = Command::new("curl")
                .args(&["-s", "-I", "debian.org"])
                .output()
                .await?;
            
            if output.status.success() {
                info!("DNS resolution test successful");
                return Ok(());
            }
            
            if attempt < max_attempts - 1 {
                debug!("DNS test failed, retrying in {} seconds...", wait_time);
                tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;
                wait_time *= 2;
            }
        }
        
        Err(FirestreamError::GeneralError("DNS resolution test failed".to_string()))
    }
    
    /// Setup port forwarding for development
    pub async fn setup_port_forwarding(&self, dev_config: &K3dDevModeConfig) -> Result<()> {
        info!("Setting up port forwarding for development");
        
        // Create logs directory
        let log_dir = self.project_root.join("logs");
        fs::create_dir_all(&log_dir).await?;
        
        // Get all services
        let output = Command::new("kubectl")
            .args(&[
                "get", "svc",
                "-n", "default",
                "-o", "jsonpath={range .items[*]}{.metadata.name},{.spec.ports[*].port}{\"\\n\"}{end}",
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(FirestreamError::GeneralError("Failed to get services".to_string()));
        }
        
        let services_str = String::from_utf8_lossy(&output.stdout);
        let mut port_forwards = Vec::new();
        
        for line in services_str.lines() {
            if line.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 2 {
                continue;
            }
            
            let service_name = parts[0];
            let ports = parts[1];
            
            for port_str in ports.split_whitespace() {
                if let Ok(internal_port) = port_str.parse::<u16>() {
                    let external_port = dev_config.port_offset + internal_port;
                    
                    info!("Forwarding service {} port {} to {}", 
                        service_name, internal_port, external_port);
                    
                    // Start port forwarding in background
                    let mut cmd = Command::new("kubectl");
                    cmd.args(&[
                        "port-forward",
                        "--namespace", "default",
                        &format!("svc/{}", service_name),
                        &format!("{}:{}", external_port, internal_port),
                    ]);
                    
                    cmd.stdout(std::process::Stdio::piped());
                    cmd.stderr(std::process::Stdio::piped());
                    
                    let _child = cmd.spawn()
                        .map_err(|e| FirestreamError::GeneralError(
                            format!("Failed to start port forward: {}", e)
                        ))?;
                    
                    port_forwards.push((service_name.to_string(), internal_port, external_port));
                }
            }
        }
        
        // Save port forward info
        let forwards_file = log_dir.join("port_forwards.json");
        let forwards_json = serde_json::to_string_pretty(&port_forwards)?;
        fs::write(&forwards_file, forwards_json).await?;
        
        info!("Port forwarding setup complete. {} ports forwarded", port_forwards.len());
        Ok(())
    }
    
    /// Delete cluster
    pub async fn delete_cluster(&self) -> Result<()> {
        info!("Deleting K3D cluster '{}'", self.config.name);
        
        let status = Command::new("k3d")
            .args(&["cluster", "delete", &self.config.name])
            .status()
            .await?;
        
        if !status.success() {
            return Err(FirestreamError::GeneralError("Failed to delete cluster".to_string()));
        }
        
        Ok(())
    }
    
    /// Get cluster info
    pub async fn get_cluster_info(&self) -> Result<HashMap<String, String>> {
        let mut info = HashMap::new();
        
        // Get cluster status
        let output = Command::new("k3d")
            .args(&["cluster", "list", "-o", "json"])
            .output()
            .await?;
        
        if output.status.success() {
            let clusters_json = String::from_utf8_lossy(&output.stdout);
            if let Ok(clusters) = serde_json::from_str::<serde_json::Value>(&clusters_json) {
                if let Some(cluster_array) = clusters.as_array() {
                    for cluster in cluster_array {
                        if cluster["name"] == self.config.name {
                            info.insert("name".to_string(), self.config.name.clone());
                            info.insert("servers".to_string(), 
                                cluster["serversCount"].as_u64().unwrap_or(0).to_string());
                            info.insert("agents".to_string(), 
                                cluster["agentsCount"].as_u64().unwrap_or(0).to_string());
                        }
                    }
                }
            }
        }
        
        // Get node info
        let output = Command::new("kubectl")
            .args(&["get", "nodes", "-o", "wide"])
            .output()
            .await?;
        
        if output.status.success() {
            info.insert("nodes".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
        }
        
        Ok(info)
    }
}

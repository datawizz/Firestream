//! K3D (K3s in Docker) provider implementation
//!
//! This module provides comprehensive K3D cluster management including:
//! - Basic cluster creation and deletion
//! - Advanced features with state integration
//! - Registry setup and management
//! - TLS certificate configuration
//! - Network configuration (routes, DNS)
//! - Development mode with port forwarding

use crate::{
    K8sManagerError, Result, K3dClusterConfig, K3dDevModeConfig, K3dConfig, CertificateConfig,
    ClusterInfo, ClusterProvider, ClusterStatus, PortForwardConfig, LogsConfig,
    DiagnosticsConfig,
};
use crate::traits::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::collections::HashMap;
use tokio::process::Command;
use tokio::fs;
use tracing::{info, debug, warn};
use base64;
use tempfile;

/// K3D cluster manager with comprehensive features
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
        
        // IMPORTANT: Patch /etc/hosts FIRST to avoid DNS resolution issues with sudo
        if self.config.network.patch_etc_hosts {
            self.patch_etc_hosts().await?;
        }
        
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
        
        // Configure network - Routes before DNS
        if self.config.network.configure_routes {
            self.setup_routes().await?;
        }
        
        if self.config.network.configure_dns {
            self.configure_dns().await?;
        }
        
        // Test DNS resolution with retries
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
            .ok_or_else(|| K8sManagerError::GeneralError("Cannot find home directory".into()))?
            .join(".kube");
        
        fs::create_dir_all(&kube_dir).await
            .map_err(|e| K8sManagerError::IoError(format!("Failed to create .kube directory: {}", e)))?;
        
        Ok(())
    }
    
    /// Reconnect to existing cluster
    async fn reconnect_cluster(&self) -> Result<()> {
        info!("Reconnecting to cluster '{}'", self.config.name);
        
        // Write kubeconfig
        let output = Command::new("k3d")
            .args(&[
                "kubeconfig",
                "write",
                &self.config.name,
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(K8sManagerError::GeneralError("Failed to write kubeconfig".to_string()));
        }
        
        // The kubeconfig path is printed to stdout
        let kubeconfig_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("Kubeconfig written to: {}", kubeconfig_path);
        
        // Set context using the kubeconfig path
        let status = Command::new("kubectl")
            .args(&[
                "config",
                "use-context",
                &format!("k3d-{}", self.config.name),
            ])
            .env("KUBECONFIG", &kubeconfig_path)
            .status()
            .await?;
        
        if !status.success() {
            warn!("Failed to set kubectl context");
        }
        
        // Test kubectl connection
        self.test_kubectl().await?;
        
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
            return Err(K8sManagerError::Timeout("Timeout waiting for cluster".to_string()));
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
            return Err(K8sManagerError::GeneralError("kubectl connection failed".to_string()));
        }
        
        let version_json = String::from_utf8_lossy(&output.stdout);
        let version: serde_json::Value = serde_json::from_str(&version_json)
            .map_err(|e| K8sManagerError::GeneralError(format!("Failed to parse kubectl version: {}", e)))?;
        
        if version["clientVersion"]["gitVersion"].is_null() || 
           version["serverVersion"]["gitVersion"].is_null() {
            return Err(K8sManagerError::GeneralError("kubectl version check failed".to_string()));
        }
        
        info!("kubectl connection verified");
        Ok(())
    }
    
    /// Generate TLS certificate
    async fn generate_tls_certificate(&self, config: &CertificateConfig) -> Result<()> {
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
    
    /// Patch /etc/hosts with hostname
    async fn patch_etc_hosts(&self) -> Result<()> {
        let hostname_output = Command::new("hostname")
            .output()
            .await?;
        
        let hostname = String::from_utf8_lossy(&hostname_output.stdout).trim().to_string();
        info!("Patching /etc/hosts for hostname: {}", hostname);
        
        // Read current /etc/hosts
        let hosts_content = fs::read_to_string("/etc/hosts").await
            .unwrap_or_default();
        
        // Check if hostname already exists
        if !hosts_content.contains(&hostname) {
            // Add hostname
            let entry = format!("127.0.0.1 {}", hostname);
            
            // Use echo with sudo to append
            let status = Command::new("sudo")
                .args(&["sh", "-c", &format!("echo '{}' >> /etc/hosts", entry)])
                .status()
                .await?;
            
            if status.success() {
                info!("Added hostname {} to /etc/hosts", hostname);
            } else {
                warn!("Failed to add hostname to /etc/hosts");
            }
        } else {
            debug!("Hostname {} already exists in /etc/hosts", hostname);
        }
        
        Ok(())
    }
    
    /// Test DNS resolution with retries
    async fn test_dns_resolution(&self) -> Result<()> {
        info!("Testing DNS resolution");
        
        // Test external DNS with curl (more reliable than nslookup)
        let external_test = Command::new("timeout")
            .args(&["2", "curl", "-s", "-I", "-m", "2", "http://google.com"])
            .output()
            .await;
        
        let external_ok = external_test.map(|o| o.status.success()).unwrap_or(false);
        
        // Test Kubernetes DNS by checking if kube-dns service exists
        let k8s_test = Command::new("kubectl")
            .args(&["get", "svc", "-n", "kube-system", "kube-dns"])
            .output()
            .await;
        
        let k8s_ok = k8s_test.map(|o| o.status.success()).unwrap_or(false);
        
        if external_ok {
            info!("External DNS resolution working");
        } else {
            return Err(K8sManagerError::DnsError(
                "External DNS resolution test failed. Check your network connection.".to_string()
            ));
        }
        
        if k8s_ok {
            info!("Kubernetes DNS service is available");
        } else {
            warn!("Kubernetes DNS service check failed");
        }
        
        Ok(())
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
            return Err(K8sManagerError::GeneralError("Failed to get services".to_string()));
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
                        .map_err(|e| K8sManagerError::GeneralError(
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
}

// Basic setup functions for convenience
/// Setup K3D cluster with default configuration
pub async fn setup_cluster() -> Result<()> {
    let config = K3dConfig::default();
    setup_cluster_with_config(&config).await
}

/// Setup K3D cluster with custom configuration
pub async fn setup_cluster_with_config(config: &K3dConfig) -> Result<()> {
    // Check if k3d is installed
    check_k3d_installed().await?;
    
    // Convert basic config to full config
    let full_config = K3dClusterConfig {
        name: config.cluster_name.clone(),
        api_port: config.api_port,
        http_port: config.lb_port,
        https_port: 443,
        servers: config.servers,
        agents: config.agents,
        registry: crate::K3dRegistryConfig {
            enabled: true,
            name: config.registry_name.clone(),
            port: config.registry_port,
        },
        ..Default::default()
    };
    
    let manager = K3dClusterManager::new(full_config);
    
    // Use simplified setup for basic config
    if manager.cluster_exists().await? {
        info!("K3D cluster '{}' already exists", config.cluster_name);
        return Ok(());
    }
    
    // Create registry
    create_registry(config).await?;
    
    // Create cluster
    create_cluster(config).await?;
    
    // Wait for cluster
    wait_for_cluster(&config.cluster_name).await?;
    
    // Configure kubectl
    configure_kubectl(&config.cluster_name).await?;
    
    info!("K3D cluster '{}' is ready", config.cluster_name);
    Ok(())
}

/// Check if k3d is installed
async fn check_k3d_installed() -> Result<()> {
    match which::which("k3d") {
        Ok(path) => {
            info!("k3d found at: {:?}", path);
            
            // Get version
            match Command::new("k3d")
                .arg("version")
                .output()
                .await
            {
                Ok(output) if output.status.success() => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    info!("k3d version: {}", version.trim());
                    Ok(())
                }
                _ => Err(K8sManagerError::GeneralError(
                    "Failed to get k3d version".to_string()
                )),
            }
        }
        Err(_) => Err(K8sManagerError::ToolNotInstalled("k3d".to_string())),
    }
}

/// Create Docker registry
async fn create_registry(config: &K3dConfig) -> Result<()> {
    // Check if registry exists
    let output = Command::new("docker")
        .args(&["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .await?;
    
    let containers = String::from_utf8_lossy(&output.stdout);
    if containers.lines().any(|name| name == config.registry_name) {
        info!("Registry '{}' already exists", config.registry_name);
        return Ok(());
    }
    
    info!("Creating Docker registry '{}'", config.registry_name);
    
    let status = Command::new("k3d")
        .args(&[
            "registry",
            "create",
            &config.registry_name,
            "--port",
            &config.registry_port.to_string(),
        ])
        .status()
        .await
        .map_err(|e| K8sManagerError::ProcessError(format!("Failed to create registry: {}", e)))?;
    
    if !status.success() {
        return Err(K8sManagerError::GeneralError(
            "Failed to create Docker registry".to_string()
        ));
    }
    
    Ok(())
}

/// Create K3D cluster
async fn create_cluster(config: &K3dConfig) -> Result<()> {
    info!("Creating K3D cluster '{}'", config.cluster_name);
    
    let mut cmd = Command::new("k3d");
    cmd.args(&[
        "cluster",
        "create",
        &config.cluster_name,
        "--api-port",
        &config.api_port.to_string(),
        "-p",
        &format!("{}:80@loadbalancer", config.lb_port),
        "--agents",
        &config.agents.to_string(),
        "--servers",
        &config.servers.to_string(),
        "--registry-use",
        &format!("{}:{}", config.registry_name, config.registry_port),
        "--k3s-arg",
        "--disable=traefik@server:*",
        "--wait",
    ]);
    
    let status = cmd.status().await
        .map_err(|e| K8sManagerError::ProcessError(format!("Failed to create cluster: {}", e)))?;
    
    if !status.success() {
        return Err(K8sManagerError::GeneralError(
            "Failed to create K3D cluster".to_string()
        ));
    }
    
    Ok(())
}

/// Wait for cluster to be ready
async fn wait_for_cluster(cluster_name: &str) -> Result<()> {
    info!("Waiting for cluster '{}' to be ready...", cluster_name);
    
    // Wait for nodes to be ready
    for i in 0..30 {
        let output = Command::new("kubectl")
            .args(&["get", "nodes", "-o", "jsonpath={.items[*].status.conditions[?(@.type=='Ready')].status}"])
            .output()
            .await?;
        
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout);
            if status.split_whitespace().all(|s| s == "True") {
                info!("All nodes are ready");
                return Ok(());
            }
        }
        
        if i < 29 {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
    
    Err(K8sManagerError::Timeout(
        "Timeout waiting for cluster to be ready".to_string()
    ))
}

/// Configure kubectl to use the cluster
async fn configure_kubectl(cluster_name: &str) -> Result<()> {
    let status = Command::new("k3d")
        .args(&["kubeconfig", "merge", cluster_name])
        .status()
        .await
        .map_err(|e| K8sManagerError::ProcessError(format!("Failed to configure kubectl: {}", e)))?;
    
    if !status.success() {
        return Err(K8sManagerError::GeneralError(
            "Failed to configure kubectl".to_string()
        ));
    }
    
    // Set as current context
    let status = Command::new("kubectl")
        .args(&["config", "use-context", &format!("k3d-{}", cluster_name)])
        .status()
        .await?;
    
    if !status.success() {
        warn!("Failed to set kubectl context");
    }
    
    Ok(())
}

/// Delete K3D cluster
pub async fn delete_cluster(cluster_name: &str) -> Result<()> {
    info!("Deleting K3D cluster '{}'", cluster_name);
    
    let status = Command::new("k3d")
        .args(&["cluster", "delete", cluster_name])
        .status()
        .await
        .map_err(|e| K8sManagerError::ProcessError(format!("Failed to delete cluster: {}", e)))?;
    
    if !status.success() {
        return Err(K8sManagerError::GeneralError(
            "Failed to delete K3D cluster".to_string()
        ));
    }
    
    Ok(())
}

// Implement ClusterManager trait
#[async_trait]
impl ClusterManager for K3dClusterManager {
    fn provider_name(&self) -> &'static str {
        "k3d"
    }
    
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
            .map_err(|e| K8sManagerError::ProcessError(format!("Failed to create cluster: {}", e)))?;
        
        if !status.success() {
            return Err(K8sManagerError::GeneralError("Failed to create cluster".to_string()));
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
    
    async fn delete_cluster(&self) -> Result<()> {
        info!("Deleting K3D cluster '{}'", self.config.name);
        
        let status = Command::new("k3d")
            .args(&["cluster", "delete", &self.config.name])
            .status()
            .await?;
        
        if !status.success() {
            return Err(K8sManagerError::GeneralError("Failed to delete cluster".to_string()));
        }
        
        Ok(())
    }
    
    async fn cluster_exists(&self) -> Result<bool> {
        let output = Command::new("k3d")
            .args(&["cluster", "list"])
            .output()
            .await
            .map_err(|e| K8sManagerError::ProcessError(format!("Failed to list clusters: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(&self.config.name))
    }
    
    async fn get_cluster_info(&self) -> Result<ClusterInfo> {
        let mut metadata = HashMap::new();
        
        // Get cluster status
        let output = Command::new("k3d")
            .args(&["cluster", "list", "-o", "json"])
            .output()
            .await?;
        
        let mut node_count = 0;
        let mut status = ClusterStatus::Unknown;
        
        if output.status.success() {
            let clusters_json = String::from_utf8_lossy(&output.stdout);
            if let Ok(clusters) = serde_json::from_str::<serde_json::Value>(&clusters_json) {
                if let Some(cluster_array) = clusters.as_array() {
                    for cluster in cluster_array {
                        if cluster["name"] == self.config.name {
                            let servers = cluster["serversCount"].as_u64().unwrap_or(0);
                            let agents = cluster["agentsCount"].as_u64().unwrap_or(0);
                            node_count = (servers + agents) as u32;
                            
                            metadata.insert("servers".to_string(), servers.to_string());
                            metadata.insert("agents".to_string(), agents.to_string());
                            
                            // Determine status (k3d doesn't provide detailed status)
                            status = ClusterStatus::Running;
                        }
                    }
                }
            }
        }
        
        // Get endpoint
        let endpoint = format!("https://localhost:{}", self.config.api_port);
        
        Ok(ClusterInfo {
            name: self.config.name.clone(),
            provider: ClusterProvider::K3d,
            status,
            endpoint: Some(endpoint),
            kubernetes_version: Some(self.config.k3s_version.clone()),
            node_count,
            metadata,
        })
    }
    
    async fn get_cluster_status(&self) -> Result<String> {
        let info = self.get_cluster_info().await?;
        Ok(format!("{:?}", info.status))
    }
    
    async fn connect_cluster(&self) -> Result<()> {
        self.reconnect_cluster().await
    }
    
    async fn update_cluster(&self) -> Result<()> {
        // K3D doesn't support in-place updates, would need to recreate
        Err(K8sManagerError::GeneralError(
            "K3D clusters cannot be updated in-place. Please delete and recreate.".to_string()
        ))
    }
}

// Implement ClusterLifecycle trait
#[async_trait]
impl ClusterLifecycle for K3dClusterManager {
    async fn start_cluster(&self) -> Result<()> {
        let status = Command::new("k3d")
            .args(&["cluster", "start", &self.config.name])
            .status()
            .await?;
        
        if !status.success() {
            return Err(K8sManagerError::GeneralError("Failed to start cluster".to_string()));
        }
        
        Ok(())
    }
    
    async fn stop_cluster(&self) -> Result<()> {
        let status = Command::new("k3d")
            .args(&["cluster", "stop", &self.config.name])
            .status()
            .await?;
        
        if !status.success() {
            return Err(K8sManagerError::GeneralError("Failed to stop cluster".to_string()));
        }
        
        Ok(())
    }
}

// Implement ClusterNetworking trait
#[async_trait]
impl ClusterNetworking for K3dClusterManager {
    async fn port_forward(&self, config: &PortForwardConfig) -> Result<()> {
        let mut cmd = Command::new("kubectl");
        cmd.args(&[
            "port-forward",
            "--namespace", &config.namespace,
            &format!("svc/{}", config.service_name),
            &format!("{}:{}", config.local_port, config.remote_port),
        ]);
        
        let _child = cmd.spawn()
            .map_err(|e| K8sManagerError::GeneralError(
                format!("Failed to start port forward: {}", e)
            ))?;
        
        Ok(())
    }
    
    async fn port_forward_all(&self, port_offset: u16) -> Result<Vec<PortForwardConfig>> {
        let dev_config = K3dDevModeConfig {
            port_forward_all: true,
            port_offset,
        };
        
        self.setup_port_forwarding(&dev_config).await?;
        
        // TODO: Return actual port forward configs
        Ok(Vec::new())
    }
    
    async fn configure_dns(&self) -> Result<()> {
        info!("Configuring DNS");
        
        // Save original resolv.conf first
        let resolv_backup = "/etc/resolv.conf.backup";
        let _ = Command::new("sudo")
            .args(&["cp", "/etc/resolv.conf", resolv_backup])
            .status()
            .await;
        
        // Read current resolv.conf
        let original_resolv = fs::read_to_string("/etc/resolv.conf").await
            .unwrap_or_else(|_| "nameserver 8.8.8.8\nnameserver 8.8.4.4".to_string());
        
        // Extract existing nameservers (skip local/k8s ones)
        let mut external_nameservers = Vec::new();
        for line in original_resolv.lines() {
            if line.starts_with("nameserver") {
                if let Some(ns) = line.split_whitespace().nth(1) {
                    if !ns.starts_with("127.") && !ns.starts_with("10.43.") && !ns.starts_with("10.42.") {
                        external_nameservers.push(ns.to_string());
                    }
                }
            }
        }
        
        // Ensure we have at least one external nameserver
        if external_nameservers.is_empty() {
            external_nameservers.push("8.8.8.8".to_string());
            external_nameservers.push("8.8.4.4".to_string());
        }
        
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
            return Err(K8sManagerError::GeneralError("Failed to get Kubernetes DNS IP".to_string()));
        }
        
        let dns_ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("Kubernetes DNS IP: {}", dns_ip);
        
        // Build new resolv.conf with external DNS first for better performance
        let mut resolv_content = String::new();
        resolv_content.push_str("search svc.cluster.local cluster.local\n");
        
        // Put external DNS first to avoid timeouts
        for ns in &external_nameservers {
            resolv_content.push_str(&format!("nameserver {}\n", ns));
        }
        
        // Then Kubernetes DNS for cluster resolution
        resolv_content.push_str(&format!("nameserver {}\n", dns_ip));
        
        resolv_content.push_str("options edns0 trust-ad\n");
        
        info!("Updating resolv.conf with Kubernetes DNS ({}) and external DNS: {:?}", dns_ip, external_nameservers);
        
        // Write new resolv.conf using cp instead of mv (for bind-mounted files)
        let temp_file = "/tmp/resolv.conf.new";
        fs::write(temp_file, &resolv_content).await?;
        
        let status = Command::new("sudo")
            .args(&["cp", temp_file, "/etc/resolv.conf"])
            .status()
            .await?;
        
        if !status.success() {
            warn!("Failed to update resolv.conf");
            return Err(K8sManagerError::NetworkError("Failed to update DNS configuration".to_string()));
        }
        
        info!("DNS configuration updated successfully");
        Ok(())
    }
    
    async fn setup_routes(&self) -> Result<()> {
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
            return Err(K8sManagerError::GeneralError("Failed to get server IP".to_string()));
        }
        
        let server_ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("K3D server IP: {}", server_ip);
        
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
}

// Implement ClusterObservability trait
#[async_trait]
impl ClusterObservability for K3dClusterManager {
    async fn get_logs(&self, config: &LogsConfig) -> Result<String> {
        let mut cmd = Command::new("kubectl");
        cmd.arg("logs");
        
        if let Some(ns) = &config.namespace {
            cmd.args(&["-n", ns]);
        }
        
        cmd.arg(format!("{}/{}", config.resource_type.as_str(), config.resource_name));
        
        if let Some(container) = &config.container {
            cmd.args(&["-c", container]);
        }
        
        if config.all_containers {
            cmd.arg("--all-containers");
        }
        
        if config.previous {
            cmd.arg("--previous");
        }
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(K8sManagerError::GeneralError(
                format!("Failed to get logs: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    async fn stream_logs(&self, config: &LogsConfig) -> Result<()> {
        let mut cmd = Command::new("kubectl");
        cmd.arg("logs");
        cmd.arg("-f"); // Follow
        
        if let Some(ns) = &config.namespace {
            cmd.args(&["-n", ns]);
        }
        
        cmd.arg(format!("{}/{}", config.resource_type.as_str(), config.resource_name));
        
        if let Some(container) = &config.container {
            cmd.args(&["-c", container]);
        }
        
        if config.all_containers {
            cmd.arg("--all-containers");
        }
        
        let mut child = cmd.spawn()?;
        let _ = child.wait().await?;
        
        Ok(())
    }
    
    async fn get_diagnostics(&self, config: &DiagnosticsConfig) -> Result<HashMap<String, String>> {
        let mut diagnostics = HashMap::new();
        
        if config.include_all || config.include_nodes {
            let output = Command::new("kubectl")
                .args(&["get", "nodes", "-o", "wide"])
                .output()
                .await?;
            
            if output.status.success() {
                diagnostics.insert("nodes".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        
        if config.include_all || config.include_pods {
            let mut cmd = Command::new("kubectl");
            cmd.args(&["get", "pods", "-o", "wide"]);
            
            if let Some(ns) = &config.namespace {
                cmd.args(&["-n", ns]);
            } else {
                cmd.arg("--all-namespaces");
            }
            
            let output = cmd.output().await?;
            
            if output.status.success() {
                diagnostics.insert("pods".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        
        if config.include_all || config.include_services {
            let mut cmd = Command::new("kubectl");
            cmd.args(&["get", "svc", "-o", "wide"]);
            
            if let Some(ns) = &config.namespace {
                cmd.args(&["-n", ns]);
            } else {
                cmd.arg("--all-namespaces");
            }
            
            let output = cmd.output().await?;
            
            if output.status.success() {
                diagnostics.insert("services".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        
        if config.include_all || config.include_events {
            let mut cmd = Command::new("kubectl");
            cmd.args(&["get", "events", "--sort-by=.metadata.creationTimestamp"]);
            
            if let Some(ns) = &config.namespace {
                cmd.args(&["-n", ns]);
            } else {
                cmd.arg("--all-namespaces");
            }
            
            let output = cmd.output().await?;
            
            if output.status.success() {
                diagnostics.insert("events".to_string(), String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        
        Ok(diagnostics)
    }
    
    async fn get_metrics(&self) -> Result<HashMap<String, serde_json::Value>> {
        // K3D doesn't have metrics server by default
        // This would need to be implemented after installing metrics-server
        Ok(HashMap::new())
    }
}

// Implement ClusterSecurity trait
#[async_trait]
impl ClusterSecurity for K3dClusterManager {
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
    
    async fn create_secret(&self, name: &str, namespace: &str, data: HashMap<String, Vec<u8>>) -> Result<()> {
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
    
    async fn get_secret(&self, name: &str, namespace: &str) -> Result<HashMap<String, Vec<u8>>> {
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

// Implement ClusterDevelopment trait
#[async_trait]
impl ClusterDevelopment for K3dClusterManager {
    async fn enable_dev_mode(&self) -> Result<()> {
        if let Some(dev_config) = &self.config.dev_mode {
            if dev_config.port_forward_all {
                self.setup_port_forwarding(dev_config).await?;
            }
        }
        Ok(())
    }
    
    async fn disable_dev_mode(&self) -> Result<()> {
        // Kill all kubectl port-forward processes
        let _ = Command::new("pkill")
            .args(&["-f", "kubectl port-forward"])
            .status()
            .await;
        
        Ok(())
    }
    
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
        
        // Create registry without --cluster flag first
        let status = Command::new("k3d")
            .args(&[
                "registry",
                "create",
                &format!("{}.{}", self.config.name, self.config.registry.name),
                "--port",
                &self.config.registry.port.to_string(),
            ])
            .status()
            .await
            .map_err(|e| K8sManagerError::ProcessError(format!("Failed to create registry: {}", e)))?;
        
        if !status.success() {
            return Err(K8sManagerError::GeneralError("Failed to create registry".to_string()));
        }
        
        Ok(())
    }
}

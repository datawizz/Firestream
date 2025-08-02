//! Networking operations for K3D clusters

use crate::{K8sManagerError, Result, K3dDevModeConfig, PortForwardConfig};
use crate::providers::k3d::manager::K3dClusterManager;
use crate::providers::k3d::utils::is_running_in_container;
use tokio::process::Command;
use tokio::fs;
use tokio::time::{sleep, Duration, timeout};
use tracing::{info, debug, warn, error};

impl K3dClusterManager {
    /// Patch /etc/hosts with hostname
    pub(crate) async fn patch_etc_hosts(&self) -> Result<()> {
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
    
    /// Test DNS resolution with proper retries
    pub async fn test_dns_resolution(&self) -> Result<()> {
        info!("Testing DNS resolution");
        
        let timeout_duration = Duration::from_secs(self.config.timeouts.dns_check);
        
        // Test external DNS
        let external_result = timeout(timeout_duration, async {
            self.retry_with_backoff("external DNS test", || async {
                let output = Command::new("curl")
                    .args(&[
                        "-s",           // Silent
                        "-I",           // Head request only
                        "-m", "5",      // 5 second timeout
                        "--connect-timeout", "3",
                        "http://google.com"
                    ])
                    .output()
                    .await?;
                
                if !output.status.success() {
                    return Err(K8sManagerError::DnsError(
                        "External DNS resolution failed".to_string()
                    ));
                }
                
                Ok(())
            }).await
        }).await;
        
        match external_result {
            Ok(_) => info!("External DNS resolution working"),
            Err(e) => {
                error!("External DNS resolution failed: {}", e);
                return Err(K8sManagerError::DnsError(
                    "External DNS resolution test failed. Check your network connection.".to_string()
                ));
            }
        }
        
        // Test Kubernetes DNS
        let k8s_dns_result = timeout(timeout_duration, async {
            self.retry_with_backoff("Kubernetes DNS test", || async {
                // First check if kube-dns service exists
                let svc_check = Command::new("kubectl")
                    .args(&["get", "svc", "-n", "kube-system", "kube-dns"])
                    .output()
                    .await?;
                
                if !svc_check.status.success() {
                    return Err(K8sManagerError::DnsError(
                        "kube-dns service not found".to_string()
                    ));
                }
                
                // Test actual DNS resolution from within the cluster
                let dns_test = Command::new("kubectl")
                    .args(&[
                        "run", "dns-test-external",
                        "--rm", "-i", "--restart=Never",
                        "--image=busybox:1.28",
                        "--",
                        "nslookup", "google.com"
                    ])
                    .output()
                    .await?;
                
                if !dns_test.status.success() {
                    return Err(K8sManagerError::DnsError(
                        "In-cluster DNS resolution failed".to_string()
                    ));
                }
                
                Ok(())
            }).await
        }).await;
        
        match k8s_dns_result {
            Ok(_) => info!("Kubernetes DNS is fully functional"),
            Err(e) => {
                warn!("Kubernetes DNS test failed: {}", e);
                // This is a warning, not a fatal error
            }
        }
        
        Ok(())
    }
    
    /// Fix kubeconfig for container environment
    pub(crate) async fn fix_kubeconfig_for_container(&self) -> Result<()> {
        if !is_running_in_container() {
            return Ok(());
        }
        
        info!("Fixing kubeconfig for container environment");
        
        // First connect to the k3d network
        self.connect_to_cluster_network().await?;
        
        // Get the k3d server's IP address on the k3d network
        let server_name = format!("k3d-{}-server-0", self.config.name);
        let network_name = format!("k3d-{}", self.config.name);
        
        let output = Command::new("docker")
            .args(&[
                "inspect",
                &server_name,
                "--format",
                &format!("{{{{(index .NetworkSettings.Networks \"{}\").IPAddress}}}}", network_name),
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(K8sManagerError::GeneralError(
                format!("Failed to get server IP for {}: {}", server_name, stderr)
            ));
        }
        
        let mut server_ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if server_ip.is_empty() || server_ip == "<no value>" {
            // Server might not be on the k3d network, try getting any IP
            let fallback_output = Command::new("docker")
                .args(&[
                    "inspect",
                    &server_name,
                    "--format",
                    "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}",
                ])
                .output()
                .await?;
            
            if fallback_output.status.success() {
                let fallback_ip = String::from_utf8_lossy(&fallback_output.stdout).trim().to_string();
                if !fallback_ip.is_empty() {
                    warn!("Server {} not found on network {}, using IP: {}", server_name, network_name, fallback_ip);
                    server_ip = fallback_ip;
                } else {
                    return Err(K8sManagerError::GeneralError(
                        format!("No IP address found for {}", server_name)
                    ));
                }
            } else {
                return Err(K8sManagerError::GeneralError(
                    format!("Failed to get any IP address for {}", server_name)
                ));
            }
        }
        
        let cluster_name = format!("k3d-{}", self.config.name);
        let new_server = format!("https://{}:6443", server_ip);
        
        info!("Updating kubeconfig to use server IP: {}", new_server);
        
        let status = Command::new("kubectl")
            .args(&[
                "config", "set-cluster",
                &cluster_name,
                &format!("--server={}", new_server),
            ])
            .status()
            .await?;
        
        if status.success() {
            info!("Updated kubeconfig server to {}", new_server);
        } else {
            return Err(K8sManagerError::GeneralError(
                "Failed to update kubeconfig server URL".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Simple fallback method to fix kubeconfig
    pub(crate) async fn fix_kubeconfig_simple(&self) -> Result<()> {
        info!("Using simple method to fix kubeconfig");
        
        // This method is now the same as the main method
        // It's kept for backward compatibility
        self.fix_kubeconfig_for_container().await
    }
    
    /// Connect the current container to the k3d network
    pub(crate) async fn connect_to_cluster_network(&self) -> Result<()> {
        info!("Connecting devcontainer to k3d network");
        
        // Get current container hostname
        let hostname = std::fs::read_to_string("/etc/hostname")
            .map_err(|e| K8sManagerError::GeneralError(format!("Failed to read hostname: {}", e)))?
            .trim()
            .to_string();
        
        let network_name = format!("k3d-{}", self.config.name);
        
        // Check if already connected
        let check = Command::new("docker")
            .args(&[
                "inspect", &hostname,
                "--format", &format!("{{{{.NetworkSettings.Networks.{}}}}}", network_name),
            ])
            .output()
            .await?;
        
        let output = String::from_utf8_lossy(&check.stdout);
        if !output.contains("<nil>") && !output.trim().is_empty() {
            info!("Already connected to k3d network");
            return Ok(());
        }
        
        // Connect to k3d network
        info!("Connecting container {} to network {}", hostname, network_name);
        let status = Command::new("docker")
            .args(&[
                "network", "connect",
                &network_name,
                &hostname,
            ])
            .status()
            .await?;
        
        if !status.success() {
            warn!("Failed to connect to k3d network - this may affect DNS resolution");
        } else {
            info!("Successfully connected to k3d network");
            // Give the network connection a moment to stabilize
            sleep(Duration::from_secs(1)).await;
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
    
    /// Get Docker internal IP for the k3d cluster
    pub(crate) async fn get_cluster_docker_ip(&self) -> Result<String> {
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
            return Err(K8sManagerError::GeneralError(
                format!("Failed to get Docker IP for cluster '{}'", self.config.name)
            ));
        }
        
        let docker_ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if docker_ip.is_empty() {
            return Err(K8sManagerError::GeneralError(
                format!("No Docker IP found for cluster '{}'", self.config.name)
            ));
        }
        
        Ok(docker_ip)
    }
    
    /// Configure DNS settings
    pub async fn configure_dns(&self) -> Result<()> {
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
        
        // Build new resolv.conf with Kubernetes DNS first for cluster resolution
        let mut resolv_content = String::new();
        resolv_content.push_str("search default.svc.cluster.local svc.cluster.local cluster.local\n");
        
        // Put Kubernetes DNS first for cluster resolution
        resolv_content.push_str(&format!("nameserver {}\n", dns_ip));
        
        // Then external DNS for non-cluster resolution
        for ns in &external_nameservers {
            resolv_content.push_str(&format!("nameserver {}\n", ns));
        }
        
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
    
    /// Setup network routes
    pub async fn setup_routes(&self) -> Result<()> {
        // Only configure routes when running in a container
        if !is_running_in_container() {
            debug!("Not running in container, skipping route setup");
            return Ok(());
        }
        
        info!("Setting up IP routes for container environment");
        
        // Find the full path to the ip command
        let ip_path = match Command::new("which")
            .arg("ip")
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            }
            _ => {
                // Try common locations
                let common_paths = [
                    "/sbin/ip",
                    "/usr/sbin/ip",
                    "/bin/ip",
                    "/usr/bin/ip",
                ];
                
                let mut found_path = None;
                for path in &common_paths {
                    if tokio::fs::metadata(path).await.is_ok() {
                        found_path = Some(path.to_string());
                        break;
                    }
                }
                
                match found_path {
                    Some(path) => path,
                    None => {
                        warn!("Could not find 'ip' command. Skipping route setup. This may affect pod/service connectivity.");
                        return Ok(());
                    }
                }
            }
        };
        
        debug!("Using ip command at: {}", ip_path);
        
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
        
        // Add routes for pod and service networks
        for cidr in [&self.config.network.pod_cidr, &self.config.network.service_cidr] {
            // Check if route exists
            let check = Command::new("sudo")
                .args(&[&ip_path, "route", "show", cidr])
                .output()
                .await?;
            
            if check.stdout.is_empty() {
                // Add route
                let status = Command::new("sudo")
                    .args(&[&ip_path, "route", "add", cidr, "via", &server_ip])
                    .status()
                    .await?;
                
                if status.success() {
                    info!("Added route for {}", cidr);
                } else {
                    return Err(K8sManagerError::NetworkError(
                        format!("Failed to add route for {}. Ensure you have sudo access", cidr)
                    ));
                }
            } else {
                debug!("Route for {} already exists", cidr);
            }
        }
        
        Ok(())
    }
    
    /// Port forward to a service
    pub async fn port_forward(&self, config: &PortForwardConfig) -> Result<()> {
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
    
    /// Port forward all services
    pub async fn port_forward_all(&self, port_offset: u16) -> Result<Vec<PortForwardConfig>> {
        let dev_config = K3dDevModeConfig {
            port_forward_all: true,
            port_offset,
        };
        
        self.setup_port_forwarding(&dev_config).await?;
        
        // TODO: Return actual port forward configs
        Ok(Vec::new())
    }
}
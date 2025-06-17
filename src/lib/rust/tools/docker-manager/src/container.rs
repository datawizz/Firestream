//! Container management operations

use crate::{DockerManager, DockerManagerError, Result};
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogsOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
    StatsOptions, Stats,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::models::{ContainerStateStatusEnum, ContainerSummary};
use futures_util::stream::StreamExt;
use tracing::{debug, info};

/// Container operations
impl DockerManager {
    /// List all containers
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>> {
        let options = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };
        
        let containers = self.client.list_containers(Some(options)).await?;
        debug!("Found {} containers", containers.len());
        Ok(containers)
    }
    
    /// Get container by name or ID
    pub async fn get_container(&self, name_or_id: &str) -> Result<ContainerSummary> {
        let containers = self.list_containers(true).await?;
        
        containers
            .into_iter()
            .find(|c| {
                c.id.as_ref().map(|id| id.starts_with(name_or_id)).unwrap_or(false) ||
                c.names.as_ref()
                    .map(|names| names.iter().any(|n| n.trim_start_matches('/') == name_or_id))
                    .unwrap_or(false)
            })
            .ok_or_else(|| DockerManagerError::ContainerNotFound(name_or_id.to_string()))
    }
    
    /// Create a new container
    pub async fn create_container(
        &self,
        name: &str,
        image: &str,
        config: Option<Config<String>>,
    ) -> Result<String> {
        let options = CreateContainerOptions {
            name,
            platform: None,
        };
        
        let config = config.unwrap_or_else(|| Config {
            image: Some(image.to_string()),
            ..Default::default()
        });
        
        let response = self.client.create_container(Some(options), config).await?;
        info!("Created container '{}' with ID: {}", name, response.id);
        Ok(response.id)
    }
    
    /// Start a container
    pub async fn start_container(&self, name_or_id: &str) -> Result<()> {
        self.client
            .start_container(name_or_id, None::<StartContainerOptions<String>>)
            .await?;
        info!("Started container '{}'", name_or_id);
        Ok(())
    }
    
    /// Stop a container
    pub async fn stop_container(&self, name_or_id: &str, timeout: Option<i64>) -> Result<()> {
        let options = StopContainerOptions {
            t: timeout.unwrap_or(10),
        };
        
        self.client.stop_container(name_or_id, Some(options)).await?;
        info!("Stopped container '{}'", name_or_id);
        Ok(())
    }
    
    /// Restart a container
    pub async fn restart_container(&self, name_or_id: &str, _timeout: Option<i64>) -> Result<()> {
        self.client.restart_container(name_or_id, None).await?;
        info!("Restarted container '{}'", name_or_id);
        Ok(())
    }
    
    /// Remove a container
    pub async fn remove_container(&self, name_or_id: &str, force: bool) -> Result<()> {
        let options = RemoveContainerOptions {
            force,
            ..Default::default()
        };
        
        self.client.remove_container(name_or_id, Some(options)).await?;
        info!("Removed container '{}'", name_or_id);
        Ok(())
    }
    
    /// Get container logs
    pub async fn container_logs(
        &self,
        name_or_id: &str,
        follow: bool,
        tail: Option<&str>,
    ) -> Result<Vec<String>> {
        let options = LogsOptions::<String> {
            follow,
            stdout: true,
            stderr: true,
            tail: tail.unwrap_or("all").to_string(),
            ..Default::default()
        };
        
        let mut stream = self.client.logs(name_or_id, Some(options));
        let mut logs = Vec::new();
        
        while let Some(Ok(output)) = stream.next().await {
            logs.push(output.to_string());
            if !follow {
                // For non-follow mode, we might want to limit the collection
                if logs.len() > 10000 {
                    break;
                }
            }
        }
        
        Ok(logs)
    }
    
    /// Execute a command in a running container
    pub async fn exec_in_container(
        &self,
        name_or_id: &str,
        cmd: Vec<&str>,
        attach: bool,
    ) -> Result<String> {
        let exec_config = CreateExecOptions {
            cmd: Some(cmd),
            attach_stdout: Some(attach),
            attach_stderr: Some(attach),
            ..Default::default()
        };
        
        let exec_instance = self.client.create_exec(name_or_id, exec_config).await?;
        
        if attach {
            let mut output = String::new();
            if let StartExecResults::Attached { output: mut stream, .. } = 
                self.client.start_exec(&exec_instance.id, None).await? 
            {
                while let Some(Ok(msg)) = stream.next().await {
                    output.push_str(&msg.to_string());
                }
            }
            Ok(output)
        } else {
            self.client.start_exec(&exec_instance.id, None).await?;
            Ok(exec_instance.id)
        }
    }
    
    /// Get container statistics
    pub async fn container_stats(&self, name_or_id: &str) -> Result<Stats> {
        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };
        
        let mut stream = self.client.stats(name_or_id, Some(options));
        
        if let Some(Ok(stats)) = stream.next().await {
            Ok(stats)
        } else {
            Err(DockerManagerError::GeneralError(
                "Failed to get container stats".to_string()
            ))
        }
    }
    
    /// Check if container is running
    pub async fn is_container_running(&self, name_or_id: &str) -> Result<bool> {
        let inspect = self.client.inspect_container(name_or_id, None).await?;
        
        Ok(inspect.state
            .and_then(|s| s.status)
            .map(|status| matches!(status, ContainerStateStatusEnum::RUNNING))
            .unwrap_or(false))
    }
    
    /// Wait for container to reach a specific state
    pub async fn wait_for_container_state(
        &self,
        name_or_id: &str,
        desired_state: ContainerStateStatusEnum,
        timeout_secs: u64,
    ) -> Result<()> {
        let start = std::time::Instant::now();
        
        loop {
            let inspect = self.client.inspect_container(name_or_id, None).await?;
            let current_state = inspect.state.and_then(|s| s.status);
            
            if let Some(state) = current_state {
                if state == desired_state {
                    return Ok(());
                }
            }
            
            if start.elapsed().as_secs() > timeout_secs {
                return Err(DockerManagerError::GeneralError(
                    format!("Timeout waiting for container '{}' to reach state {:?}", 
                        name_or_id, desired_state)
                ));
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
}
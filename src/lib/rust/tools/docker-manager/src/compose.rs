//! Docker Compose operations

use crate::{DockerManager, DockerManagerError, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, info};

/// Docker Compose operations
impl DockerManager {
    /// Run a docker-compose command
    pub async fn compose_command(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
        env_vars: Option<Vec<(&str, &str)>>,
    ) -> Result<String> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose");
        
        for arg in args {
            cmd.arg(arg);
        }
        
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        
        if let Some(vars) = env_vars {
            for (key, value) in vars {
                cmd.env(key, value);
            }
        }
        
        debug!("Running docker compose command: {:?}", cmd);
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DockerManagerError::GeneralError(
                format!("Docker compose command failed: {}", stderr)
            ));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Run docker-compose up
    pub async fn compose_up(
        &self,
        working_dir: &Path,
        detach: bool,
        services: Option<Vec<&str>>,
    ) -> Result<()> {
        let mut args = vec!["up"];
        
        if detach {
            args.push("-d");
        }
        
        if let Some(services) = services {
            args.extend(services);
        }
        
        self.compose_command(&args, Some(working_dir), None).await?;
        info!("Started docker-compose services in {}", working_dir.display());
        Ok(())
    }
    
    /// Run docker-compose down
    pub async fn compose_down(
        &self,
        working_dir: &Path,
        remove_volumes: bool,
        remove_orphans: bool,
    ) -> Result<()> {
        let mut args = vec!["down"];
        
        if remove_volumes {
            args.push("-v");
        }
        
        if remove_orphans {
            args.push("--remove-orphans");
        }
        
        self.compose_command(&args, Some(working_dir), None).await?;
        info!("Stopped docker-compose services in {}", working_dir.display());
        Ok(())
    }
    
    /// Run docker-compose logs
    pub async fn compose_logs(
        &self,
        working_dir: &Path,
        follow: bool,
        tail: Option<&str>,
        services: Option<Vec<&str>>,
    ) -> Result<String> {
        let mut args = vec!["logs"];
        
        if follow {
            args.push("-f");
        }
        
        if let Some(tail) = tail {
            args.push("--tail");
            args.push(tail);
        }
        
        if let Some(services) = services {
            args.extend(services);
        }
        
        self.compose_command(&args, Some(working_dir), None).await
    }
    
    /// Stream docker-compose logs
    pub async fn compose_logs_stream<F>(
        &self,
        working_dir: &Path,
        services: Option<Vec<&str>>,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(String) + Send,
    {
        let mut cmd = Command::new("docker");
        cmd.arg("compose")
            .arg("logs")
            .arg("-f")
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        if let Some(services) = services {
            for service in services {
                cmd.arg(service);
            }
        }
        
        let mut child = cmd.spawn()?;
        
        let stdout = child.stdout.take()
            .ok_or_else(|| DockerManagerError::IoError("Failed to get stdout".to_string()))?;
        let stderr = child.stderr.take()
            .ok_or_else(|| DockerManagerError::IoError("Failed to get stderr".to_string()))?;
        
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);
        
        let mut stdout_lines = stdout_reader.lines();
        let mut stderr_lines = stderr_reader.lines();
        
        loop {
            tokio::select! {
                line = stdout_lines.next_line() => {
                    match line {
                        Ok(Some(line)) => callback(line),
                        Ok(None) => break,
                        Err(e) => return Err(e.into()),
                    }
                }
                line = stderr_lines.next_line() => {
                    match line {
                        Ok(Some(line)) => callback(line),
                        Ok(None) => break,
                        Err(e) => return Err(e.into()),
                    }
                }
            }
        }
        
        let _ = child.wait().await?;
        Ok(())
    }
    
    /// Run docker-compose ps
    pub async fn compose_ps(&self, working_dir: &Path) -> Result<String> {
        self.compose_command(&["ps"], Some(working_dir), None).await
    }
    
    /// Run docker-compose restart
    pub async fn compose_restart(
        &self,
        working_dir: &Path,
        services: Option<Vec<&str>>,
    ) -> Result<()> {
        let mut args = vec!["restart"];
        
        if let Some(services) = services {
            args.extend(services);
        }
        
        self.compose_command(&args, Some(working_dir), None).await?;
        info!("Restarted docker-compose services in {}", working_dir.display());
        Ok(())
    }
    
    /// Run docker-compose build
    pub async fn compose_build(
        &self,
        working_dir: &Path,
        no_cache: bool,
        services: Option<Vec<&str>>,
    ) -> Result<()> {
        let mut args = vec!["build"];
        
        if no_cache {
            args.push("--no-cache");
        }
        
        if let Some(services) = services {
            args.extend(services);
        }
        
        self.compose_command(&args, Some(working_dir), None).await?;
        info!("Built docker-compose services in {}", working_dir.display());
        Ok(())
    }
    
    /// Validate docker-compose file
    pub async fn compose_config(&self, working_dir: &Path) -> Result<String> {
        self.compose_command(&["config"], Some(working_dir), None).await
    }
    
    /// Run docker-compose pull
    pub async fn compose_pull(
        &self,
        working_dir: &Path,
        services: Option<Vec<&str>>,
    ) -> Result<()> {
        let mut args = vec!["pull"];
        
        if let Some(services) = services {
            args.extend(services);
        }
        
        self.compose_command(&args, Some(working_dir), None).await?;
        info!("Pulled docker-compose images in {}", working_dir.display());
        Ok(())
    }
}
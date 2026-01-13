//! Docker test runner
//!
//! Runs containers and tests using Docker.

use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::{CreateImageOptions, ListImagesOptions};
use bollard::models::HostConfig;
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::default::Default;

/// Docker test runner
pub struct DockerRunner {
    /// Docker client
    client: Docker,
}

impl DockerRunner {
    /// Create a new Docker runner
    pub fn new() -> Result<Self, super::Error> {
        let client = Docker::connect_with_local_defaults()
            .map_err(|e| super::Error::Docker(e.to_string()))?;
        Ok(Self { client })
    }

    /// Create a Docker runner with a custom socket
    pub fn with_socket(socket: &str) -> Result<Self, super::Error> {
        let client = Docker::connect_with_socket(socket, 120, bollard::API_DEFAULT_VERSION)
            .map_err(|e| super::Error::Docker(e.to_string()))?;
        Ok(Self { client })
    }

    /// Run a container and execute tests
    pub async fn run_container(
        &self,
        image: &str,
        env: &HashMap<String, String>,
    ) -> Result<String, super::Error> {
        // Ensure the image exists locally
        self.ensure_image(image).await?;

        // Convert environment HashMap to Vec<String> in "KEY=VALUE" format
        let env_vec: Vec<String> = env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Create container configuration
        let config = Config {
            image: Some(image.to_string()),
            env: Some(env_vec),
            host_config: Some(HostConfig {
                auto_remove: Some(false), // Don't auto-remove so we can cleanup manually
                ..Default::default()
            }),
            tty: Some(true), // Keep container running
            ..Default::default()
        };

        // Create the container
        let container = self
            .client
            .create_container(None::<CreateContainerOptions<String>>, config)
            .await
            .map_err(|e| super::Error::Docker(format!("Failed to create container: {}", e)))?;

        // Start the container
        self.client
            .start_container::<String>(&container.id, None)
            .await
            .map_err(|e| {
                super::Error::Docker(format!("Failed to start container {}: {}", container.id, e))
            })?;

        Ok(container.id)
    }

    /// Execute a command in a running container
    pub async fn exec(
        &self,
        container_id: &str,
        cmd: Vec<String>,
    ) -> Result<String, super::Error> {
        // Create exec instance
        let exec = self
            .client
            .create_exec(
                container_id,
                CreateExecOptions {
                    cmd: Some(cmd.clone()),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| {
                super::Error::Docker(format!(
                    "Failed to create exec in container {}: {}",
                    container_id, e
                ))
            })?;

        // Start exec and capture output
        match self.client.start_exec(&exec.id, None).await {
            Ok(StartExecResults::Attached { mut output, .. }) => {
                let mut result = String::new();

                // Collect output from stream
                while let Some(chunk) = output.next().await {
                    match chunk {
                        Ok(log_output) => {
                            result.push_str(&log_output.to_string());
                        }
                        Err(e) => {
                            return Err(super::Error::Docker(format!(
                                "Error reading exec output: {}",
                                e
                            )));
                        }
                    }
                }

                Ok(result)
            }
            Ok(StartExecResults::Detached) => {
                Err(super::Error::Docker("Exec started in detached mode".to_string()))
            }
            Err(e) => Err(super::Error::Docker(format!(
                "Failed to start exec in container {}: {}",
                container_id, e
            ))),
        }
    }

    /// Stop and remove a container
    pub async fn cleanup(&self, container_id: &str) -> Result<(), super::Error> {
        // Stop the container (wait up to 10 seconds)
        if let Err(e) = self
            .client
            .stop_container(container_id, Some(StopContainerOptions { t: 10 }))
            .await
        {
            // Log the error but continue with removal
            tracing::warn!("Failed to stop container {}: {}", container_id, e);
        }

        // Remove the container (force removal)
        self.client
            .remove_container(
                container_id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
            .map_err(|e| {
                super::Error::Docker(format!("Failed to remove container {}: {}", container_id, e))
            })?;

        Ok(())
    }

    /// Pull an image if not present
    pub async fn ensure_image(&self, image: &str) -> Result<(), super::Error> {
        // Parse image name and tag
        let (image_name, tag) = if let Some(pos) = image.rfind(':') {
            (&image[..pos], &image[pos + 1..])
        } else {
            (image, "latest")
        };

        // Check if image already exists
        let mut filters_map = HashMap::new();
        filters_map.insert("reference".to_string(), vec![image.to_string()]);

        let options = Some(ListImagesOptions {
            filters: filters_map,
            ..Default::default()
        });

        let images = self
            .client
            .list_images(options)
            .await
            .map_err(|e| super::Error::Docker(format!("Failed to list images: {}", e)))?;

        // If image exists, we're done
        if !images.is_empty() {
            tracing::info!("Image {} already exists locally", image);
            return Ok(());
        }

        // Pull the image
        tracing::info!("Pulling image {}...", image);
        let options = CreateImageOptions {
            from_image: image_name,
            tag,
            ..Default::default()
        };

        let mut stream = self.client.create_image(Some(options), None, None);

        // Process the pull stream
        while let Some(info) = stream.next().await {
            match info {
                Ok(info) => {
                    if let Some(status) = info.status {
                        tracing::debug!("Pull status: {}", status);
                    }
                    if let Some(error) = info.error {
                        return Err(super::Error::Docker(format!("Image pull error: {}", error)));
                    }
                }
                Err(e) => {
                    return Err(super::Error::Docker(format!("Failed to pull image: {}", e)));
                }
            }
        }

        tracing::info!("Successfully pulled image {}", image);
        Ok(())
    }
}

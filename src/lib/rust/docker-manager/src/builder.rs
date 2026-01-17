//! Docker image building operations

use crate::{DockerManager, DockerManagerError, Result};
use bollard::image::BuildImageOptions;
use bollard::models::BuildInfo;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

/// Build operations
impl DockerManager {
    /// Build an image from a Dockerfile
    pub async fn build_image(
        &self,
        context_path: &Path,
        dockerfile: Option<&str>,
        tag: &str,
        build_args: Option<HashMap<String, String>>,
        no_cache: bool,
    ) -> Result<String> {
        // Create tar archive of the build context
        let tar_data = self.create_build_context(context_path)?;
        
        let dockerfile = dockerfile.unwrap_or("Dockerfile");
        
        let buildargs: HashMap<&str, &str> = build_args
            .as_ref()
            .map(|args| args.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect())
            .unwrap_or_default();
        
        let options = BuildImageOptions {
            dockerfile,
            t: tag,
            buildargs,
            nocache: no_cache,
            rm: true, // Remove intermediate containers
            forcerm: true, // Always remove intermediate containers
            ..Default::default()
        };
        
        info!("Building image '{}' from {}", tag, context_path.display());
        
        let mut stream = self.client.build_image(options, None, Some(tar_data.into()));
        let mut image_id = None;
        
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(build_info) => {
                    if let Some(stream_msg) = build_info.stream {
                        debug!("{}", stream_msg.trim());
                    }
                    if let Some(aux) = build_info.aux {
                        if let Some(id) = aux.id {
                            image_id = Some(id);
                        }
                    }
                    if let Some(error) = build_info.error {
                        return Err(DockerManagerError::BuildError(error));
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        
        match image_id {
            Some(id) => {
                info!("Successfully built image '{}' with ID: {}", tag, id);
                Ok(id)
            }
            None => {
                // If we didn't get an ID, try to find the image by tag
                self.get_image(tag).await
                    .map(|img| img.id)
                    .map_err(|_| DockerManagerError::BuildError("Build completed but image ID not found".to_string()))
            }
        }
    }
    
    /// Build image with progress callback
    pub async fn build_image_with_progress<F>(
        &self,
        context_path: &Path,
        dockerfile: Option<&str>,
        tag: &str,
        build_args: Option<HashMap<String, String>>,
        no_cache: bool,
        mut progress_callback: F,
    ) -> Result<String>
    where
        F: FnMut(BuildProgress) + Send,
    {
        let tar_data = self.create_build_context(context_path)?;
        
        let dockerfile = dockerfile.unwrap_or("Dockerfile");
        
        let buildargs: HashMap<&str, &str> = build_args
            .as_ref()
            .map(|args| args.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect())
            .unwrap_or_default();
        
        let options = BuildImageOptions {
            dockerfile,
            t: tag,
            buildargs,
            nocache: no_cache,
            rm: true,
            forcerm: true,
            ..Default::default()
        };
        
        info!("Building image '{}' from {}", tag, context_path.display());
        
        let mut stream = self.client.build_image(options, None, Some(tar_data.into()));
        let mut image_id = None;
        
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(build_info) => {
                    let progress = self.parse_build_info(&build_info);
                    progress_callback(progress);
                    
                    if let Some(aux) = build_info.aux {
                        if let Some(id) = aux.id {
                            image_id = Some(id);
                        }
                    }
                    if let Some(error) = build_info.error {
                        return Err(DockerManagerError::BuildError(error));
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        
        match image_id {
            Some(id) => {
                info!("Successfully built image '{}' with ID: {}", tag, id);
                Ok(id)
            }
            None => {
                self.get_image(tag).await
                    .map(|img| img.id)
                    .map_err(|_| DockerManagerError::BuildError("Build completed but image ID not found".to_string()))
            }
        }
    }
    
    /// Create a tar archive of the build context
    fn create_build_context(&self, context_path: &Path) -> Result<Vec<u8>> {
        use tar::Builder;
        
        if !context_path.exists() {
            return Err(DockerManagerError::IoError(
                format!("Build context path does not exist: {}", context_path.display())
            ));
        }
        
        let mut tar_data = Vec::new();
        {
            let mut tar = Builder::new(&mut tar_data);
            
            // Check for .dockerignore
            let dockerignore_path = context_path.join(".dockerignore");
            let ignore_patterns = if dockerignore_path.exists() {
                self.parse_dockerignore(&dockerignore_path)?
            } else {
                vec![]
            };
            
            // Add files to tar
            self.add_to_tar(&mut tar, context_path, context_path, &ignore_patterns)?;
            
            tar.finish()?;
        }
        
        Ok(tar_data)
    }
    
    /// Parse .dockerignore file
    fn parse_dockerignore(&self, path: &Path) -> Result<Vec<String>> {
        use std::fs;
        use std::io::{BufRead, BufReader};
        
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let patterns: Vec<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .collect();
        
        Ok(patterns)
    }
    
    /// Add files to tar archive
    fn add_to_tar(
        &self,
        tar: &mut tar::Builder<&mut Vec<u8>>,
        base_path: &Path,
        current_path: &Path,
        ignore_patterns: &[String],
    ) -> Result<()> {
        use std::fs;
        
        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(base_path)
                .map_err(|e| DockerManagerError::IoError(e.to_string()))?;
            
            // Check if path should be ignored
            let should_ignore = ignore_patterns.iter().any(|pattern| {
                relative_path.to_string_lossy().contains(pattern)
            });
            
            if should_ignore {
                continue;
            }
            
            if path.is_dir() {
                self.add_to_tar(tar, base_path, &path, ignore_patterns)?;
            } else {
                tar.append_path_with_name(&path, relative_path)?;
            }
        }
        
        Ok(())
    }
    
    /// Parse build info into progress
    fn parse_build_info(&self, info: &BuildInfo) -> BuildProgress {
        BuildProgress {
            step: info.stream.clone(),
            error: info.error.clone(),
            status: info.status.clone(),
            progress: info.progress.clone(),
        }
    }
}

/// Build progress information
#[derive(Debug, Clone)]
pub struct BuildProgress {
    pub step: Option<String>,
    pub error: Option<String>,
    pub status: Option<String>,
    pub progress: Option<String>,
}
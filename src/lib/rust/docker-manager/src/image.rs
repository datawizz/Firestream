//! Image management operations

use crate::{DockerManager, DockerManagerError, Result};
use bollard::image::{
    CreateImageOptions, ListImagesOptions, PushImageOptions, RemoveImageOptions,
    TagImageOptions, SearchImagesOptions,
};
use bollard::models::{ImageSummary, ImageSearchResponseItem};
use bollard::auth::DockerCredentials;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use tracing::{debug, info};

/// Image operations
impl DockerManager {
    /// List all images
    pub async fn list_images(&self, all: bool) -> Result<Vec<ImageSummary>> {
        let options = ListImagesOptions::<String> {
            all,
            ..Default::default()
        };
        
        let images = self.client.list_images(Some(options)).await?;
        debug!("Found {} images", images.len());
        Ok(images)
    }
    
    /// Get image by name or ID
    pub async fn get_image(&self, name_or_id: &str) -> Result<ImageSummary> {
        let images = self.list_images(true).await?;
        
        images
            .into_iter()
            .find(|img| {
                if img.id == name_or_id {
                    return true;
                }
                img.repo_tags.iter().any(|tag| tag == name_or_id || tag.starts_with(&format!("{}:", name_or_id)))
            })
            .ok_or_else(|| DockerManagerError::ImageNotFound(name_or_id.to_string()))
    }
    
    /// Pull an image from registry
    pub async fn pull_image(&self, image: &str, tag: Option<&str>) -> Result<()> {
        let tag = tag.unwrap_or("latest");
        let options = CreateImageOptions {
            from_image: image,
            tag,
            ..Default::default()
        };
        
        info!("Pulling image {}:{}", image, tag);
        let mut stream = self.client.create_image(Some(options), None, None);
        
        // Process the stream to handle progress
        while let Some(info) = stream.next().await {
            match info {
                Ok(output) => {
                    if let Some(status) = output.status {
                        debug!("{}", status);
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        
        info!("Successfully pulled {}:{}", image, tag);
        Ok(())
    }
    
    /// Push an image to registry
    pub async fn push_image(
        &self,
        image: &str,
        tag: Option<&str>,
        credentials: Option<DockerCredentials>,
    ) -> Result<()> {
        let tag = tag.unwrap_or("latest");
        let options = PushImageOptions {
            tag,
        };
        
        info!("Pushing image {}:{}", image, tag);
        let mut stream = self.client.push_image(image, Some(options), credentials);
        
        while let Some(info) = stream.next().await {
            match info {
                Ok(output) => {
                    if let Some(status) = output.status {
                        debug!("Push status: {}", status);
                    }
                    if let Some(progress) = output.progress {
                        debug!("Push progress: {}", progress);
                    }
                    // Check for errors in the output
                    if let Some(error) = output.error {
                        return Err(DockerManagerError::DockerApiError(
                            format!("Push failed: {}", error)
                        ));
                    }
                }
                Err(e) => {
                    // Check if it's an auth error
                    let error_str = e.to_string();
                    if error_str.contains("authentication") || error_str.contains("unauthorized") {
                        return Err(DockerManagerError::DockerApiError(
                            format!("Authentication required for registry push: {}", e)
                        ));
                    }
                    return Err(DockerManagerError::DockerApiError(
                        format!("Docker push error: {}", e)
                    ));
                }
            }
        }
        
        info!("Successfully pushed {}:{}", image, tag);
        Ok(())
    }
    
    /// Remove an image
    pub async fn remove_image(&self, name_or_id: &str, force: bool) -> Result<Vec<String>> {
        let options = RemoveImageOptions {
            force,
            ..Default::default()
        };
        
        let results = self.client.remove_image(name_or_id, Some(options), None).await?;
        
        let removed: Vec<String> = results
            .iter()
            .filter_map(|r| r.deleted.clone())
            .collect();
        
        info!("Removed image '{}', {} layers deleted", name_or_id, removed.len());
        Ok(removed)
    }
    
    /// Tag an image
    pub async fn tag_image(&self, source: &str, target: &str, tag: &str) -> Result<()> {
        let options = TagImageOptions {
            repo: target,
            tag,
        };
        
        self.client.tag_image(source, Some(options)).await?;
        info!("Tagged image '{}' as '{}:{}'", source, target, tag);
        Ok(())
    }
    
    /// Search for images on Docker Hub
    pub async fn search_images(&self, term: &str, limit: Option<u64>) -> Result<Vec<ImageSearchResponseItem>> {
        let options = SearchImagesOptions {
            term: term.to_string(),
            limit,
            ..Default::default()
        };
        
        let results = self.client.search_images(options).await?;
        debug!("Found {} images matching '{}'", results.len(), term);
        Ok(results)
    }
    
    /// Get image history
    pub async fn image_history(&self, name_or_id: &str) -> Result<Vec<bollard::models::HistoryResponseItem>> {
        let history = self.client.image_history(name_or_id).await?;
        Ok(history)
    }
    
    /// Inspect an image
    pub async fn inspect_image(&self, name_or_id: &str) -> Result<bollard::models::ImageInspect> {
        let inspect = self.client.inspect_image(name_or_id).await?;
        Ok(inspect)
    }
    
    /// Prune unused images
    pub async fn prune_images(&self, all: bool) -> Result<(u64, Vec<String>)> {
        let mut filters = HashMap::new();
        if !all {
            filters.insert("dangling", vec!["true"]);
        }
        
        let options = bollard::image::PruneImagesOptions {
            filters,
        };
        
        let result = self.client.prune_images(Some(options)).await?;
        
        let space_reclaimed = result.space_reclaimed.unwrap_or(0) as u64;
        let images_deleted = result.images_deleted
            .unwrap_or_default()
            .into_iter()
            .filter_map(|img| img.deleted)
            .collect();
        
        info!("Pruned images: {} bytes reclaimed", space_reclaimed);
        Ok((space_reclaimed, images_deleted))
    }
    
    /// Check if image exists locally
    pub async fn image_exists(&self, name: &str) -> bool {
        self.get_image(name).await.is_ok()
    }
}
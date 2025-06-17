//! Basic usage example for docker-manager library

use docker_manager::{DockerManager, utils};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("docker_manager=info")
        .init();
    
    // Create Docker manager instance
    let docker = DockerManager::new().await?;
    
    println!("Connected to Docker daemon!");
    println!();
    
    // Check if we're running in Docker
    if DockerManager::is_running_in_docker() {
        println!("Running inside a Docker container");
    } else {
        println!("Running on host system");
    }
    println!();
    
    // List containers
    println!("=== Containers ===");
    let containers = docker.list_containers(true).await?;
    for container in containers {
        let name = container.names
            .as_ref()
            .and_then(|n| n.first())
            .map(|n| n.trim_start_matches('/'))
            .unwrap_or("unnamed");
        let status = container.status.as_deref().unwrap_or("unknown");
        println!("  {} - {}", name, status);
    }
    println!();
    
    // List images
    println!("=== Images ===");
    let images = docker.list_images(false).await?;
    for image in images.iter().take(5) {
        for tag in &image.repo_tags {
            let size = utils::format_bytes(image.size);
            println!("  {} ({})", tag, size);
        }
    }
    if images.len() > 5 {
        println!("  ... and {} more images", images.len() - 5);
    }
    println!();
    
    // Show system info
    println!("=== System Info ===");
    let info = docker.info().await?;
    println!("  Containers: {} ({} running)", 
        info.containers.unwrap_or(0),
        info.containers_running.unwrap_or(0)
    );
    println!("  Images: {}", info.images.unwrap_or(0));
    println!("  Server Version: {}", info.server_version.unwrap_or_default());
    println!("  Storage Driver: {}", info.driver.unwrap_or_default());
    println!();
    
    // Check disk usage
    println!("=== Disk Usage ===");
    let usage = docker.disk_usage().await?;
    
    if let Some(images) = usage.images {
        let total_size: i64 = images.iter().map(|i| i.size).sum();
        println!("  Images: {} using {}", images.len(), utils::format_bytes(total_size));
    }
    
    if let Some(containers) = usage.containers {
        let size: i64 = containers.iter()
            .filter_map(|c| c.size_rw)
            .sum();
        println!("  Containers: {} using {}", containers.len(), utils::format_bytes(size));
    }
    
    if let Some(volumes) = usage.volumes {
        let size: i64 = volumes.iter()
            .filter_map(|v| v.usage_data.as_ref()
                .map(|u| u.size))
            .sum();
        println!("  Volumes: {} using {}", volumes.len(), utils::format_bytes(size));
    }
    
    Ok(())
}
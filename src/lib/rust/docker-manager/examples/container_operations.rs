//! Example of container operations using docker-manager

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
    
    // Example: Pull an image if it doesn't exist
    let test_image = "busybox:latest";
    if !docker.image_exists(test_image).await {
        println!("Pulling image {}...", test_image);
        docker.pull_image("busybox", Some("latest")).await?;
    }
    
    // Example: Create and run a container
    let container_name = format!("test-container-{}", chrono::Utc::now().timestamp());
    
    println!("Creating container '{}'...", container_name);
    let container_id = docker.create_container(
        &container_name,
        test_image,
        Some(bollard::container::Config {
            image: Some(test_image.to_string()),
            cmd: Some(vec!["sh".to_string(), "-c".to_string(), "echo 'Hello from Docker!' && sleep 10".to_string()]),
            ..Default::default()
        }),
    ).await?;
    
    println!("Starting container...");
    docker.start_container(&container_id).await?;
    
    // Example: Execute a command in the running container
    println!("Executing command in container...");
    let exec_output = docker.exec_in_container(
        &container_id,
        vec!["ls", "-la", "/"],
        true,
    ).await?;
    println!("Command output:\n{}", exec_output);
    
    // Example: Get container logs
    println!("\nContainer logs:");
    let logs = docker.container_logs(&container_id, false, Some("10")).await?;
    for log in logs {
        print!("{}", log);
    }
    
    // Example: Get container stats
    if docker.is_container_running(&container_id).await? {
        let stats = docker.container_stats(&container_id).await?;
        let cpu_percent = utils::calculate_cpu_percentage(&stats);
        let (mem_usage, mem_percent) = utils::calculate_memory_usage(&stats);
        
        println!("\nContainer stats:");
        println!("  CPU: {:.2}%", cpu_percent);
        println!("  Memory: {} ({:.2}%)", utils::format_bytes(mem_usage as i64), mem_percent);
    }
    
    // Example: Stop and remove the container
    println!("\nStopping container...");
    docker.stop_container(&container_id, Some(5)).await?;
    
    println!("Removing container...");
    docker.remove_container(&container_id, false).await?;
    
    println!("\nContainer operations completed!");
    
    Ok(())
}
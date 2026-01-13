//! Example of building Docker images using docker-manager

use docker_manager::DockerManager;
use anyhow::Result;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("docker_manager=debug")
        .init();
    
    // Create Docker manager instance
    let docker = DockerManager::new().await?;
    
    // Create a temporary directory for our build context
    let temp_dir = tempfile::tempdir()?;
    let context_path = temp_dir.path();
    
    // Create a simple Dockerfile
    let dockerfile_content = r#"
FROM alpine:latest
RUN apk add --no-cache curl
WORKDIR /app
COPY hello.txt .
CMD ["cat", "hello.txt"]
"#;
    
    fs::write(context_path.join("Dockerfile"), dockerfile_content)?;
    
    // Create a file to copy
    fs::write(context_path.join("hello.txt"), "Hello from Docker build example!\n")?;
    
    // Build the image
    let tag = format!("docker-manager-example:{}", chrono::Utc::now().timestamp());
    
    println!("Building image '{}' from {}", tag, context_path.display());
    println!("Build context files:");
    for entry in fs::read_dir(context_path)? {
        let entry = entry?;
        println!("  - {}", entry.file_name().to_string_lossy());
    }
    
    let image_id = docker.build_image_with_progress(
        context_path,
        None, // Use default Dockerfile
        &tag,
        None, // No build args
        false, // Use cache
        |progress| {
            if let Some(step) = &progress.step {
                print!("{}", step);
            }
            if let Some(error) = &progress.error {
                eprintln!("Build error: {}", error);
            }
        },
    ).await?;
    
    println!("\nSuccessfully built image!");
    println!("Image ID: {}", image_id);
    println!("Tag: {}", tag);
    
    // Verify the image exists
    if docker.image_exists(&tag).await {
        println!("\nImage details:");
        let image = docker.get_image(&tag).await?;
        println!("  Size: {}", humansize::format_size(image.size as u64, humansize::BINARY));
        
        // Get image history
        println!("\nImage history:");
        let history = docker.image_history(&tag).await?;
        for (i, item) in history.iter().take(5).enumerate() {
            let created_by = item.created_by.chars().take(50).collect::<String>();
            println!("  {}: {}", i + 1, created_by);
        }
    }
    
    // Run a container from the built image
    println!("\nRunning container from built image...");
    let container_name = format!("test-build-{}", chrono::Utc::now().timestamp());
    let container_id = docker.create_container(&container_name, &tag, None).await?;
    
    docker.start_container(&container_id).await?;
    
    // Get the output
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let logs = docker.container_logs(&container_id, false, None).await?;
    println!("\nContainer output:");
    for log in logs {
        print!("{}", log);
    }
    
    // Cleanup
    println!("\nCleaning up...");
    docker.remove_container(&container_id, true).await?;
    docker.remove_image(&tag, false).await?;
    
    println!("Build example completed!");
    
    Ok(())
}
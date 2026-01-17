//! Example: Deploy a complete application stack

use helm_manager::{HelmManager, Stack};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create helm manager
    let manager = HelmManager::new().await?;

    // Define application stack
    let stack = Stack::builder()
        .name("firestream-stack")
        .namespace("default")
        .env_file("/workspace/etc/.env")
        
        // Database layer
        .add("postgresql", |d| d
            .name("database")
            .env_prefix("POSTGRES_")
            .set_value("persistence.size", serde_json::json!("10Gi"))
            .set_value("metrics.enabled", serde_json::json!(true)))
        
        // Cache layer
        .add("redis", |d| d
            .name("cache")
            .env_prefix("REDIS_")
            .depends_on("database"))
        
        // Message queue
        .add("kafka", |d| d
            .name("streaming")
            .env_prefix("KAFKA_")
            .set_value("kraft.enabled", serde_json::json!(true))
            .depends_on("database"))
        
        // Search engine
        .add("elasticsearch", |d| d
            .name("search")
            .env_prefix("ELASTICSEARCH_")
            .set_value("coordinating.replicaCount", serde_json::json!(2)))
        
        .build()?;

    // Validate dependencies
    stack.validate_dependencies()?;
    
    println!("Deploying stack: {}", stack.name);
    println!("Components:");
    for deployment in &stack.deployments {
        println!("  - {} ({})", deployment.name, deployment.chart);
    }

    // Deploy the stack
    let releases = manager.deploy_stack(stack).await?;
    
    println!("\nDeployment complete!");
    for release in releases {
        println!("  ✓ {} ({}): {}", release.name, release.chart, release.status);
    }

    Ok(())
}
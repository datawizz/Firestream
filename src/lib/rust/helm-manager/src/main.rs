use helm_manager::HelmManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create helm manager
    let manager = HelmManager::new().await?;

    // List available charts
    println!("Available Bitnami Charts:");
    for chart in manager.list_charts() {
        println!("  - {}", chart);
    }

    // Example: Deploy PostgreSQL
    // let deployment = Deployment::builder("postgresql")
    //     .name("my-database")
    //     .namespace("default")
    //     .env_file("/workspace/etc/.env")
    //     .build()?;
    //
    // let release = manager.deploy(deployment).await?;
    // println!("Deployed: {} in namespace {}", release.name, release.namespace);

    Ok(())
}

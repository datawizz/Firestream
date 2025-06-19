//! Example: Deploy PostgreSQL with environment configuration

use helm_manager::{HelmManager, Deployment, providers::bitnami::BitnamiProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create helm manager
    let manager = HelmManager::new().await?;

    // Get production values for PostgreSQL
    let production_values = BitnamiProvider::get_production_values("postgresql")?;

    // Create deployment configuration
    let deployment = Deployment::builder("postgresql")
        .name("firestream-db")
        .namespace("default")
        .env_file("/workspace/etc/.env")
        .env_prefix("POSTGRES_")
        .values(production_values)
        .set_value("persistence.size", serde_json::json!("20Gi"))
        .set_value("replicaCount", serde_json::json!(3))
        .wait()
        .atomic()
        .timeout(600)
        .build()?;

    println!("Deploying PostgreSQL with configuration:");
    println!("  Name: {}", deployment.name);
    println!("  Namespace: {}", deployment.namespace);
    println!("  Environment file: {:?}", deployment.env_file);
    println!("  Wait for ready: {}", deployment.wait);
    println!("  Atomic deployment: {}", deployment.atomic);

    // Deploy PostgreSQL
    let release = manager.deploy(deployment).await?;
    
    println!("\nDeployment successful!");
    println!("  Release: {}", release.name);
    println!("  Chart: {} v{}", release.chart, release.chart_version);
    println!("  Status: {}", release.status);
    println!("  Revision: {}", release.revision);

    // Get release status
    let status = manager.status(&release.name).await?;
    println!("\nRelease status: {}", status);

    Ok(())
}
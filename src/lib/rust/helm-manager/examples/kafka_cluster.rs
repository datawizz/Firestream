//! Example: Deploy a Kafka cluster with custom configuration

use helm_manager::{HelmManager, Deployment, Values};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create helm manager
    let manager = HelmManager::new().await?;

    // Create custom values for Kafka
    let mut kafka_values = Values::new();
    
    // Enable KRaft mode (no Zookeeper)
    kafka_values.set("kraft.enabled", json!(true));
    
    // Configure brokers
    kafka_values.set("broker.replicaCount", json!(3));
    kafka_values.set("broker.persistence.enabled", json!(true));
    kafka_values.set("broker.persistence.size", json!("50Gi"));
    
    // Configure resources
    kafka_values.set("broker.resources.requests.memory", json!("2Gi"));
    kafka_values.set("broker.resources.requests.cpu", json!("1000m"));
    kafka_values.set("broker.resources.limits.memory", json!("4Gi"));
    kafka_values.set("broker.resources.limits.cpu", json!("2000m"));
    
    // Enable metrics
    kafka_values.set("metrics.kafka.enabled", json!(true));
    kafka_values.set("metrics.jmx.enabled", json!(true));
    kafka_values.set("metrics.serviceMonitor.enabled", json!(true));

    // Create deployment
    let deployment = Deployment::builder("kafka")
        .name("streaming-platform")
        .namespace("data-pipeline")
        .env_file("/workspace/etc/.env")
        .values(kafka_values)
        .values_file("/workspace/kafka-production.yaml")  // Additional values file
        .wait()
        .timeout(900)  // 15 minutes for Kafka cluster
        .build()?;

    println!("Deploying Kafka cluster...");
    println!("  Configuration:");
    println!("  - KRaft mode enabled (no Zookeeper)");
    println!("  - 3 broker replicas");
    println!("  - 50Gi storage per broker");
    println!("  - Metrics and monitoring enabled");

    // Check if namespace exists, create if not
    // (In real implementation, kubectl_client would handle this)
    
    // Deploy Kafka
    let release = manager.deploy(deployment).await?;
    
    println!("\nKafka cluster deployed!");
    println!("  Release: {}", release.name);
    println!("  Status: {}", release.status);

    // List all releases to verify
    println!("\nAll releases:");
    let releases = manager.list_releases().await?;
    for release in releases {
        println!("  - {} ({}) in {}: {}", 
            release.name, 
            release.chart, 
            release.namespace,
            release.status
        );
    }

    Ok(())
}
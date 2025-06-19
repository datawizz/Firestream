# Helm Manager

A Rust library for managing Helm charts with embedded Bitnami charts. This crate provides a functional interface for deploying and managing the full lifecycle of Helm resources.

## Features

- **Embedded Bitnami Charts**: All Bitnami charts are embedded at compile time for deterministic deployments
- **Environment-based Configuration**: Configure charts using `.env` files and environment variables
- **Values File Support**: Layer multiple `values.yaml` files with proper precedence
- **Builder Pattern API**: Intuitive deployment configuration using builder patterns
- **Stack Deployments**: Deploy multiple related charts with dependency management
- **Async/Await**: Full async support using Tokio
- **Type Safety**: Strongly typed models for charts, releases, and values

## Requirements

- Rust 2024 edition
- `helm` and `kubectl` installed and available in PATH
- Access to a Kubernetes cluster

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
helm-manager = { path = "../helm-manager" }
```

## Usage

### Basic Deployment

```rust
use helm_manager::{HelmManager, Deployment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = HelmManager::new().await?;
    
    let deployment = Deployment::builder("postgresql")
        .name("my-database")
        .namespace("default")
        .env_file("/workspace/etc/.env")
        .build()?;
    
    let release = manager.deploy(deployment).await?;
    println!("Deployed: {}", release.name);
    Ok(())
}
```

### Stack Deployment

Deploy multiple related services:

```rust
use helm_manager::{HelmManager, Stack};

let stack = Stack::builder()
    .env_file("/workspace/etc/.env")
    .add("postgresql", |d| d.name("database"))
    .add("redis", |d| d.name("cache"))
    .add("kafka", |d| d
        .name("streaming")
        .depends_on("database"))
    .build()?;

let releases = manager.deploy_stack(stack).await?;
```

### Configuration Sources

Values are resolved in the following precedence order (highest to lowest):

1. Inline values set via builder
2. Environment variables
3. Values files (in order specified)
4. Default chart values

### Environment Variable Mapping

The library automatically maps common environment variables to Helm values:

- `POSTGRES_USER` → `postgresql.auth.username`
- `POSTGRES_PASSWORD` → `postgresql.auth.password`
- `KAFKA_BOOTSTRAP_SERVERS` → `kafka.bootstrapServers`
- `REDIS_PASSWORD` → `redis.auth.password`

For custom prefixes:

```rust
let deployment = Deployment::builder("postgresql")
    .env_prefix("MYAPP_DB_")  // MYAPP_DB_PASSWORD → password
    .build()?;
```

### Advanced Configuration

```rust
use helm_manager::{Deployment, Values};
use serde_json::json;

// Create custom values
let mut values = Values::new();
values.set("persistence.size", json!("100Gi"));
values.set("replicaCount", json!(3));

let deployment = Deployment::builder("postgresql")
    .name("production-db")
    .namespace("production")
    .env_file("/etc/production/.env")
    .values_file("/etc/production/postgres-values.yaml")
    .values(values)
    .atomic()  // Rollback on failure
    .wait()    // Wait for pods to be ready
    .timeout(600)  // 10 minute timeout
    .build()?;
```

## Available Charts

The following Bitnami charts are embedded:

- PostgreSQL
- MySQL/MariaDB
- MongoDB
- Redis
- Elasticsearch
- Kafka
- Cassandra
- Nginx
- Apache
- And many more...

List all available charts:

```rust
let charts = manager.list_charts();
for chart in charts {
    println!("- {}", chart);
}
```

## Development

### Building

The build process embeds Bitnami charts from `$HOME/bitnami-charts`:

```bash
cargo build --release
```

### Testing

```bash
cargo test
cargo test --features integration_tests  # Requires running k8s cluster
```

### Examples

See the `examples/` directory for more usage examples:

- `deploy_stack.rs` - Deploy a complete application stack
- `postgresql_setup.rs` - PostgreSQL with production configuration
- `kafka_cluster.rs` - Kafka cluster setup

## Architecture

The library consists of several key components:

- **EmbeddedCharts**: Manages extraction and caching of embedded chart data
- **HelmClient**: Wrapper around the helm CLI
- **ValuesResolver**: Resolves values from multiple sources with proper precedence
- **DeploymentBuilder**: Fluent API for configuring deployments
- **ChartManager**: Manages chart lifecycle and metadata

## License

MIT
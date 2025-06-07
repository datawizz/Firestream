# K8s Manager

A Rust library for managing Kubernetes clusters across multiple providers.

## Overview

The `k8s_manager` crate provides a unified interface for managing Kubernetes clusters across different providers, including local development clusters (k3d) and cloud providers (GKE, EKS, AKS).

## Installation

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
k8s_manager = { path = "path/to/k8s_manager" }
```

### As a CLI Tool

```bash
# Install the CLI
cargo install --path .

# Or run directly
cargo run -- --help
```

## Features

- **Unified Interface**: Common traits for all cluster operations
- **Multiple Providers**: Support for k3d (implemented), GKE, EKS, AKS (planned)
- **Comprehensive Management**: Cluster lifecycle, networking, observability, security, and development features
- **Type-Safe Configuration**: Strongly typed configuration for each provider
- **Async/Await**: Fully async API using Tokio

## Architecture

### Traits

The library defines several traits that cluster managers can implement:

- `ClusterManager`: Core cluster operations (create, delete, info)
- `ClusterLifecycle`: Start, stop, restart operations
- `ClusterNetworking`: Port forwarding, DNS, routing
- `ClusterObservability`: Logs, diagnostics, metrics
- `ClusterSecurity`: TLS, secrets management
- `ClusterDevelopment`: Development mode, local registry

### Providers

#### K3D (Implemented)
Local Kubernetes clusters using k3d (k3s in Docker).

Location: `src/providers/k3d.rs`

Features:
- Automatic registry setup
- TLS certificate generation  
- Network configuration (routes, DNS)
- Development mode with automatic port forwarding
- State integration
- Complete trait implementations for all manager traits

#### GKE (Planned)
Google Kubernetes Engine integration.

#### EKS (Planned)
Amazon Elastic Kubernetes Service integration.

#### AKS (Planned)
Azure Kubernetes Service integration.

## Usage

### Command Line Interface

The k8s-manager CLI provides easy access to all functionality:

```bash
# Create a cluster
k8s-manager cluster create my-cluster

# Setup a fully configured cluster
k8s-manager cluster setup

# View cluster info
k8s-manager cluster info my-cluster

# Delete a cluster
k8s-manager cluster delete my-cluster
```

See [CLI.md](CLI.md) for comprehensive CLI documentation.

### Library Usage

#### Basic K3D Setup

```rust
use k8s_manager::k3d;

// Simple cluster setup with defaults
k3d::setup_cluster().await?;

// Custom configuration
let config = k8s_manager::K3dConfig {
    cluster_name: "my-cluster".to_string(),
    api_port: 6443,
    lb_port: 8080,
    agents: 2,
    servers: 1,
    ..Default::default()
};
k3d::setup_cluster_with_config(&config).await?;
```

### Advanced K3D Management

```rust
use k8s_manager::{K3dClusterManager, K3dClusterConfig, ClusterManager};

// Create manager with full configuration
let config = K3dClusterConfig::default();
let manager = K3dClusterManager::new(config);

// Setup complete cluster with all features
manager.setup_cluster().await?;

// Use trait methods
manager.create_cluster().await?;
manager.port_forward_all(10000).await?;
manager.get_diagnostics(&DiagnosticsConfig::default()).await?;
```

## Configuration

### K3D Configuration

```toml
[cluster.k3d]
name = "firestream-local"
api_port = 6550
http_port = 80
https_port = 443
servers = 1
agents = 2
k3s_version = "v1.31.2-k3s1"

[cluster.k3d.registry]
enabled = true
name = "registry.localhost"
port = 5000

[cluster.k3d.tls]
enabled = true
secret_name = "firestream-tls"

[cluster.k3d.network]
configure_routes = true
configure_dns = true
patch_etc_hosts = true
pod_cidr = "10.42.0.0/16"
service_cidr = "10.43.0.0/16"

[cluster.k3d.dev_mode]
port_forward_all = true
port_offset = 10000
```

## Development

### Running Tests

See [TESTING.md](TESTING.md) for comprehensive testing guide.

```bash
# Run unit tests
cargo test

# Run integration tests (requires Docker and k3d)
cargo test --features integration_tests
```

### Adding a New Provider

1. Create a new module in `src/providers/` (e.g., `src/providers/myprovider.rs`)
2. Implement the required traits for your provider
3. Add provider-specific configuration types
4. Update `src/providers/mod.rs` to include your provider
5. Update `src/lib.rs` to re-export your provider
6. Add tests in `tests/myprovider_tests.rs`

Example structure:
```rust
// src/providers/myprovider.rs
use crate::{Result, ClusterInfo, traits::*};
use async_trait::async_trait;

pub struct MyProviderManager {
    config: MyProviderConfig,
}

#[async_trait]
impl ClusterManager for MyProviderManager {
    fn provider_name(&self) -> &'static str {
        "myprovider"
    }
    
    async fn create_cluster(&self) -> Result<()> {
        // Implementation
    }
    
    // Implement other required methods
}

// Implement other traits as needed
```

See `src/providers/k3d.rs` for a complete implementation example.

## Error Handling

The library uses a custom `K8sManagerError` type with variants for different error scenarios:

- `IoError`: File system operations
- `ProcessError`: External command execution
- `ConfigError`: Configuration parsing
- `ClusterAlreadyExists`: Cluster name conflicts
- `ClusterNotFound`: Missing clusters
- `ToolNotInstalled`: Missing required tools
- `Timeout`: Operation timeouts
- `NetworkError`: Network configuration issues
- `TlsError`: Certificate problems

## Dependencies

- `tokio`: Async runtime
- `async-trait`: Async traits
- `serde`: Serialization
- `tracing`: Logging
- `which`: Tool detection
- `dirs`: System directories

## License

See the main Firestream project license.

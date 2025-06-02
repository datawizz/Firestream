# Helm Lifecycle Management in Firestream

This document explains how to use the Helm lifecycle management system in Firestream, which provides comprehensive lifecycle hooks for Helm chart deployments.

## Overview

The Helm lifecycle system in Firestream is inspired by Qovery's approach but adapted for Firestream's architecture. It provides:

- **Trait-based extensibility**: Define custom behavior for any Helm chart
- **Comprehensive lifecycle hooks**: Pre-exec, exec, post-exec, validation, and failure handling
- **State integration**: Works seamlessly with Firestream's plan/apply workflow
- **Built-in chart implementations**: Common charts with sensible defaults

## Architecture

### Core Components

1. **HelmChart Trait**: The main interface that all charts implement
2. **ChartInfo**: Configuration structure for Helm charts
3. **Lifecycle Methods**: Hooks for different stages of deployment
4. **Built-in Charts**: Pre-configured implementations for common services

### Lifecycle Flow

```
check_prerequisites → pre_exec → exec → post_exec → validate
                                    ↓
                              on_deploy_failure (on error)
```

## Using the Helm Lifecycle System

### 1. Using Built-in Charts

Firestream includes several pre-configured charts:

```rust
use firestream::deploy::helm_lifecycle::{
    PrometheusOperatorChart,
    ExternalDnsChart,
    NginxIngressChart,
    PostgresqlChart,
    KafkaChart,
};

// Create charts with defaults
let prometheus = PrometheusOperatorChart::default();
let nginx = NginxIngressChart::k3d_local();
let postgres = PostgresqlChart::development();
```

### 2. Creating Custom Charts

You can create custom chart implementations by implementing the `HelmChart` trait:

```rust
use firestream::deploy::helm_lifecycle::{HelmChart, ChartInfo, ChartPayload, ValidationResult};
use async_trait::async_trait;

pub struct MyCustomChart {
    pub chart_info: ChartInfo,
}

#[async_trait]
impl HelmChart for MyCustomChart {
    fn get_chart_info(&self) -> &ChartInfo {
        &self.chart_info
    }
    
    fn get_chart_info_mut(&mut self) -> &mut ChartInfo {
        &mut self.chart_info
    }
    
    // Implement custom lifecycle methods...
}
```

### 3. Lifecycle Methods

#### check_prerequisites
Validates that all requirements are met before deployment:
- Values files exist and are readable
- Dependencies are satisfied
- Required resources are available

#### pre_exec
Runs before the main deployment:
- Add Helm repositories
- Create prerequisite resources
- Modify existing resources for compatibility

#### exec
The main deployment action:
- Deploy: `helm upgrade --install`
- Destroy: `helm uninstall`
- Skip: No operation

#### post_exec
Runs after successful deployment:
- Wait for resources to be ready
- Create additional resources
- Configure integrations

#### validate
Ensures the deployment was successful:
- Check pod readiness
- Verify service endpoints
- Test connectivity

#### on_deploy_failure
Handles deployment failures:
- Collect debug information
- Attempt recovery
- Clean up partial deployments

## Configuration

### ChartInfo Structure

```rust
pub struct ChartInfo {
    pub name: String,                    // Release name
    pub repository: Option<String>,      // Helm repo name
    pub chart: String,                   // Chart name
    pub version: Option<String>,         // Chart version
    pub namespace: HelmChartNamespace,   // Target namespace
    pub action: HelmAction,              // Deploy/Destroy/Skip
    pub timeout: String,                 // Deployment timeout
    pub values: Vec<ChartSetValue>,      // --set values
    pub values_files: Vec<PathBuf>,      // -f values files
    // ... more options
}
```

### Example: PostgreSQL with Custom Configuration

```rust
let mut postgres = PostgresqlChart::default();
let info = postgres.get_chart_info_mut();

// Add custom values
info.values.push(ChartSetValue {
    key: "auth.database".to_string(),
    value: "myapp".to_string(),
});

// Add values file
info.values_files.push(PathBuf::from("values/postgres-custom.yaml"));

// Set custom timeout
info.timeout = "600s".to_string();
```

## Integration with State Management

The Helm lifecycle system is fully integrated with Firestream's state management:

```toml
# In firestream.toml
[[deployments.postgresql]]
chart = "bitnami/postgresql"
version = "13.2.24"
namespace = "default"

[deployments.postgresql.values]
auth.database = "firestream"
auth.postgresPassword = "secure-password"
primary.persistence.size = "10Gi"
```

When you run `firestream plan`, it will:
1. Detect the deployment configuration
2. Create appropriate `HelmChart` instances
3. Generate execution plan with lifecycle support

## Advanced Features

### Breaking Version Handling

Some charts require special handling for major version upgrades:

```rust
let breaking_version = BreakingVersion {
    version: "2.0.0".to_string(),
    requires_uninstall: true,
    pre_upgrade_commands: vec![
        "kubectl delete crd my-old-crd.example.com".to_string(),
    ],
};

chart_info.last_breaking_version_requiring_restart = Some(breaking_version);
```

### Custom Validation

Implement sophisticated validation logic:

```rust
async fn validate(
    &self,
    kubernetes_config: &Path,
    envs: &[(String, String)],
    payload: Option<ChartPayload>,
) -> Result<Vec<ValidationResult>> {
    let mut results = vec![];
    
    // Check custom endpoints
    let endpoint_check = self.check_service_endpoint().await?;
    results.push(endpoint_check);
    
    // Verify data integrity
    let data_check = self.verify_data_integrity().await?;
    results.push(data_check);
    
    Ok(results)
}
```

### Chart Dependencies

Specify dependencies between charts:

```rust
chart_info.depends_on = vec!["postgresql".to_string(), "redis".to_string()];
```

## Examples

### Example 1: Prometheus with CRD Management

See `src/deploy/helm_lifecycle/charts/prometheus.rs` for a complete example that:
- Manages CRDs lifecycle
- Handles breaking changes
- Validates Prometheus instances
- Cleans up on uninstall

### Example 2: NGINX Ingress for K3D

See `src/deploy/helm_lifecycle/charts/nginx.rs` for an example that:
- Configures for local K3D clusters
- Validates ingress class availability
- Checks LoadBalancer status

### Example 3: Kafka with Strimzi Operator

See `src/deploy/helm_lifecycle/charts/kafka.rs` for an example that:
- Deploys the Strimzi operator
- Manages Kafka CRDs
- Provides post-deployment instructions

## Best Practices

1. **Always implement validation**: Ensure your deployments are working correctly
2. **Handle failures gracefully**: Collect debug information in `on_deploy_failure`
3. **Use prerequisites check**: Validate requirements before attempting deployment
4. **Document breaking changes**: Make major version upgrades smooth
5. **Leverage the payload**: Pass data between lifecycle stages

## Troubleshooting

### Enable Debug Logging

```bash
RUST_LOG=debug firestream apply
```

### Check Lifecycle Execution

The system logs each lifecycle stage:
```
INFO Running lifecycle for chart: prometheus-operator
INFO Checking prerequisites for chart: prometheus-operator
INFO Pre-exec: Adding prometheus-community Helm repository
INFO Deploying Helm chart: prometheus-operator
INFO Post-exec: Waiting for Prometheus Operator to be ready
INFO Validation 'release_exists': PASSED - Helm release 'prometheus-operator' exists
```

### Common Issues

1. **"Values file not found"**: Ensure paths in `values_files` are relative to the project root
2. **"Failed to add repository"**: Check internet connectivity and repository URLs
3. **"Validation failed"**: Review the validation results for specific failure reasons

## Migration from Standard Helm

To migrate existing Helm deployments:

1. Create a `ChartInfo` matching your current deployment
2. Implement any custom lifecycle logic needed
3. Use `firestream import deployment <name>` to import existing releases
4. Future deployments will use the lifecycle system

## Future Enhancements

- Helm hooks integration
- Automatic rollback on validation failure
- Parallel chart deployment
- Chart template rendering
- OCI registry support

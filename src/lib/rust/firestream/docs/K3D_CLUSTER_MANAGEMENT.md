# K3D Cluster Management in Firestream

This document describes the integrated K3D cluster management functionality in Firestream, which provides an opinionated, state-managed approach to creating and managing local Kubernetes clusters.

## Overview

Firestream now includes comprehensive K3D cluster management that replaces shell scripts with a state-driven, reproducible approach. All cluster configuration is tracked in Firestream's state, enabling consistent cluster setup and management.

## Features

### Core Functionality
- **State-Driven**: All cluster configuration tracked in Firestream state
- **Registry Management**: Automatic local container registry setup
- **TLS Configuration**: Self-signed certificate generation and management
- **Network Configuration**: Automatic IP routes, DNS, and /etc/hosts setup
- **Development Mode**: Automatic port forwarding with configurable offset

### Advanced Features
- **Clean Mode**: Delete and recreate clusters
- **Multi-Node Support**: Configure server and agent node counts
- **Custom K3s Versions**: Specify exact K3s image versions
- **Integrated with Plan/Apply**: Works with Firestream's state workflow

## CLI Usage

### Create a Cluster

```bash
# Create with defaults
firestream cluster create

# Create with custom name and node count
firestream cluster create --name my-cluster --servers 1 --agents 3

# Create with development mode (auto port-forwarding)
firestream cluster create --dev-mode

# Clean mode - delete existing and recreate
firestream cluster create --clean

# Create from configuration file
firestream cluster create --config k3d-cluster.toml
```

### Delete a Cluster

```bash
# Delete by name
firestream cluster delete my-cluster

# Delete default cluster
firestream cluster delete
```

### Get Cluster Info

```bash
# Show info for specific cluster
firestream cluster info my-cluster

# Show info for default cluster
firestream cluster info
```

### Port Forwarding

```bash
# Forward all services with default offset (10000)
firestream cluster port-forward

# Forward with custom offset
firestream cluster port-forward --offset 20000

# Forward specific service (coming soon)
firestream cluster port-forward my-service
```

### View Logs

```bash
# View logs from all pods in default namespace
firestream cluster logs

# View logs from specific pod
firestream cluster logs pod my-pod

# View logs from deployment
firestream cluster logs deployment my-deployment

# Follow logs from a service
firestream cluster logs service my-service --follow

# View logs from all containers in a pod
firestream cluster logs pod my-pod --all-containers

# View previous container logs
firestream cluster logs pod my-pod --previous

# View logs from specific namespace
firestream cluster logs -n kube-system
```

### Cluster Diagnostics

```bash
# Get all diagnostic information
firestream cluster diagnostics --all

# Get specific diagnostics
firestream cluster diagnostics --nodes --pods
firestream cluster diagnostics --services --events

# Get diagnostics for specific namespace
firestream cluster diagnostics --pods -n default

# Get diagnostics for all namespaces
firestream cluster diagnostics --pods -n all
```

## Configuration File

Create a `firestream.toml` file:

```toml
[project]
name = "my-firestream-project"
environment = "development"

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

[cluster.k3d.tls.certificate]
country = "US"
state = "New Mexico"
locality = "Roswell"
organization = "ACME Co, LLC."
organizational_unit = "ACME Department"
common_name = "*.firestream.local"
email = "admin@firestream.local"

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

## State Integration

The K3D cluster configuration is fully integrated with Firestream's state management:

### Plan/Apply Workflow

```bash
# Edit firestream.toml with k3d configuration
# Generate plan
firestream plan

# Review changes
# Apply plan
firestream apply
```

### State Structure

The cluster configuration is stored in the state file:

```json
{
  "cluster": {
    "name": "firestream-local",
    "cluster_type": "k3d",
    "managed": true,
    "k3d_config": {
      "name": "firestream-local",
      "api_port": 6550,
      // ... full configuration
    }
  }
}
```

## Network Configuration Details

### IP Routes
Automatically configures routes for:
- Pod CIDR: 10.42.0.0/16
- Service CIDR: 10.43.0.0/16

### DNS Configuration
- Updates /etc/resolv.conf to use Kubernetes DNS
- Adds search domains for cluster services
- Tests DNS resolution after setup

### TLS Certificates
- Generates self-signed certificates
- Stores in `~/certs/`
- Creates Kubernetes secret for ingress

## Development Mode

When `dev_mode` is enabled:
- Automatically forwards all service ports
- Adds configurable offset (default 10000)
- Logs port mappings to `logs/port_forwards.json`

Example: Service on port 8080 → localhost:18080

## Comparison with Shell Scripts

| Feature | Shell Scripts | Firestream |
|---------|--------------|------------|
| State Management | ❌ None | ✅ Full state tracking |
| Reproducibility | ❌ Manual | ✅ Automatic |
| Error Handling | ❌ Basic | ✅ Comprehensive |
| Integration | ❌ Standalone | ✅ Integrated with services |
| Rollback | ❌ Manual | ✅ State-based |
| Multi-cluster | ❌ Complex | ✅ State-managed |

## Troubleshooting

### Common Issues

1. **"k3d is not installed"**
   - Install k3d: `curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash`

2. **"Failed to create registry"**
   - Check if Docker is running
   - Ensure ports are not in use

3. **"DNS resolution test failed"**
   - Check network connectivity
   - Verify firewall settings

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug firestream cluster create
```

### Logs

- TUI logs: `.firestream/tui.log`
- Port forwarding: `logs/port_forwards.json`

## Migration from Shell Scripts

For existing k3d clusters created with shell scripts:

1. Import existing cluster:
   ```bash
   firestream import cluster firestream-local
   ```

2. Update configuration in `firestream.toml`

3. Use Firestream for future management

## Future Enhancements

- [ ] Multiple cluster profiles
- [ ] Cluster templates
- [ ] Resource monitoring integration
- [ ] Automatic service discovery
- [ ] GitOps integration
- [ ] Remote cluster support

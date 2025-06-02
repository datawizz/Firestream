# K8s Manager CLI

The k8s-manager CLI provides command-line access to all cluster management functionality.

## Installation

From the k8s_manager directory:

```bash
cargo install --path .
```

Or run directly with cargo:

```bash
cargo run -- <command>
```

## Usage

```bash
k8s-manager [OPTIONS] <COMMAND>
```

### Global Options

- `-v, --verbose` - Increase verbosity (can be used multiple times)
- `-p, --provider <PROVIDER>` - Provider to use (default: k3d)
- `-h, --help` - Print help information

## Commands

### Cluster Management

#### Create a cluster

```bash
# Basic cluster with defaults
k8s-manager cluster create

# Custom cluster configuration
k8s-manager cluster create my-cluster \
  --api-port 6551 \
  --http-port 8080 \
  --servers 3 \
  --agents 2

# Development cluster with port forwarding
k8s-manager cluster create dev-cluster \
  --dev-mode \
  --port-offset 30000

# Minimal cluster without extra features
k8s-manager cluster create minimal \
  --basic \
  --no-tls \
  --no-network
```

#### Setup a cluster (full featured)

```bash
# Setup with all features (create if needed, configure everything)
k8s-manager cluster setup

# Setup with custom name
k8s-manager cluster setup production-cluster

# Quick setup with all defaults
k8s-manager cluster setup --defaults
```

#### Delete a cluster

```bash
k8s-manager cluster delete my-cluster
```

#### List clusters

```bash
k8s-manager cluster list
```

#### Get cluster information

```bash
k8s-manager cluster info my-cluster
```

#### Lifecycle operations

```bash
# Stop a running cluster
k8s-manager cluster stop my-cluster

# Start a stopped cluster
k8s-manager cluster start my-cluster

# Restart a cluster
k8s-manager cluster restart my-cluster
```

### Registry Management

```bash
# Create a registry
k8s-manager registry create --port 5001

# Create a named registry
k8s-manager registry create my-registry --port 5002

# Delete a registry
k8s-manager registry delete my-registry

# List registries
k8s-manager registry list
```

### Port Forwarding

```bash
# Forward a service port
k8s-manager port-forward my-service --local-port 8080 --remote-port 80

# Forward from a specific namespace
k8s-manager port-forward my-service \
  --namespace production \
  --local-port 9090 \
  --remote-port 8080
```

### Logs

```bash
# View pod logs
k8s-manager logs my-pod

# View deployment logs
k8s-manager logs my-deployment --resource-type deployment

# Follow logs
k8s-manager logs my-pod --follow

# View logs from a specific container
k8s-manager logs my-pod --container nginx

# View previous container logs
k8s-manager logs my-pod --previous

# View logs from all containers
k8s-manager logs my-pod --all-containers

# View logs from a specific namespace
k8s-manager logs my-pod --namespace kube-system
```

### Diagnostics

```bash
# Get all diagnostics
k8s-manager diagnostics --all

# Get specific diagnostics
k8s-manager diagnostics --nodes --pods --services

# Get diagnostics for a specific namespace
k8s-manager diagnostics --namespace production --pods --services

# Get cluster events
k8s-manager diagnostics --events
```

## Examples

### Complete Workflow

```bash
# 1. Create a development cluster
k8s-manager cluster create dev-cluster \
  --dev-mode \
  --port-offset 20000

# 2. Check cluster status
k8s-manager cluster info dev-cluster

# 3. View cluster resources
k8s-manager diagnostics --all

# 4. Deploy your application
kubectl apply -f my-app.yaml

# 5. Forward a service port
k8s-manager port-forward my-app-service \
  --local-port 8080 \
  --remote-port 80

# 6. View application logs
k8s-manager logs my-app --follow

# 7. Stop the cluster when done
k8s-manager cluster stop dev-cluster

# 8. Delete the cluster
k8s-manager cluster delete dev-cluster
```

### Quick Development Setup

```bash
# One command to create a fully configured development cluster
k8s-manager cluster setup dev --defaults

# Port forward multiple services
k8s-manager port-forward web --local-port 8080 --remote-port 80 &
k8s-manager port-forward api --local-port 8081 --remote-port 8080 &
```

### Production-like Setup

```bash
# Create a multi-node cluster
k8s-manager cluster create prod-test \
  --servers 3 \
  --agents 3 \
  --api-port 6443 \
  --https-port 443

# Get detailed diagnostics
k8s-manager diagnostics --all --namespace default
```

## Environment Variables

- `RUST_LOG` - Set log level (trace, debug, info, warn, error)

```bash
# Enable debug logging
RUST_LOG=debug k8s-manager cluster create

# Enable trace logging for specific modules
RUST_LOG=k8s_manager=trace k8s-manager cluster setup
```

## Shell Completion

Generate shell completions:

```bash
# Bash
k8s-manager --generate-completion bash > /etc/bash_completion.d/k8s-manager

# Zsh
k8s-manager --generate-completion zsh > ~/.zsh/completion/_k8s-manager

# Fish
k8s-manager --generate-completion fish > ~/.config/fish/completions/k8s-manager.fish
```

## Tips

1. **Use verbose mode** for troubleshooting: `k8s-manager -vv cluster create`

2. **Check cluster state** before operations: `k8s-manager cluster list`

3. **Use basic mode** for quick testing: `k8s-manager cluster create test --basic`

4. **Set up aliases** for common commands:
   ```bash
   alias k8sm='k8s-manager'
   alias k8sm-create='k8s-manager cluster create'
   alias k8sm-delete='k8s-manager cluster delete'
   ```

5. **Combine with kubectl** for full control:
   ```bash
   k8s-manager cluster create my-cluster
   kubectl config use-context k3d-my-cluster
   kubectl get nodes
   ```

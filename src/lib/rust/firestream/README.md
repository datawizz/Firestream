# Firestream CLI/TUI

A Kubernetes deployment platform with state-based plan/apply workflow, providing both command-line and interactive text-based UI for managing infrastructure and services.

## Features

### MVP Implementation (✅ Completed)

- **Dual Interface**: Both CLI and TUI interfaces
- **Service Management**: Install, start, stop, and check status of services
- **Configuration Management**: Strongly-typed TOML configuration files
- **Service Discovery**: List available and installed services
- **Basic Monitoring**: View logs and resource usage
- **Offline Mode**: Works without Kubernetes connection for testing
- **K3D Cluster Management**: State-managed local Kubernetes clusters with advanced networking

## Installation

```bash
cd src/lib/rust/firestream
cargo build --release
```

## Usage

### Plan/Apply Workflow

Firestream uses a state-based approach similar to Terraform:

```bash
# Initialize a new project
firestream init --name my-project
cd my-project

# Edit firestream.toml to configure your infrastructure and services

# Generate an execution plan
firestream plan

# Apply the changes
firestream apply

# Apply without confirmation
firestream apply --auto-approve

# Target specific resources
firestream plan --target deployment.postgresql
firestream apply --target deployment.postgresql
```

### State Management

```bash
# Show current state
firestream state show

# Lock/unlock state
firestream state lock
firestream state unlock

# Import existing resources
firestream import deployment my-existing-app --data resource.json

# Refresh state from actual resources
firestream refresh
```

### CLI Mode

```bash
# Install a service
firestream install kafka --config examples/kafka.toml

# Start/stop services
firestream start kafka
firestream stop kafka

# Check status
firestream status
firestream status kafka

# List services
firestream list --available
firestream list

# View logs
firestream logs kafka --follow

# Show resource usage
firestream resources
```

### Cluster Management

```bash
# Create a k3d cluster with defaults
firestream cluster create

# Create with configuration file
firestream cluster create --config examples/k3d-cluster.toml

# Create with development mode (auto port-forwarding)
firestream cluster create --dev-mode

# Delete cluster
firestream cluster delete

# Get cluster info
firestream cluster info

# Setup port forwarding
firestream cluster port-forward
```

### TUI Mode

```bash
# Launch TUI
firestream tui
# or just
firestream
```

**TUI Key Bindings:**
- `↑/k`, `↓/j` - Navigate services
- `Tab`, `Shift+Tab` - Switch tabs
- `i` - Install service
- `s` - Start/stop service
- `r` - Restart service
- `l` - View logs
- `?` - Help
- `q` - Quit

## Configuration

### Directory Structure
```
~/.firestream/
├── config.toml           # Global configuration
├── services/            # Service-specific configs
│   ├── kafka.toml
│   ├── postgresql.toml
│   └── ...
└── state/               # Runtime state
    └── services.json    # Current service states
```

### Example Service Configuration

See `examples/kafka.toml` and `examples/postgresql.toml` for complete examples.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│   CLI Parser    │     │    TUI (ratatui)│
│   (clap)        │     │                 │
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │
              ┌──────▼──────┐
              │  Core Logic │
              │   (Rust)    │
              └──────┬──────┘
                     │
         ┌───────────┼───────────┐
         │           │           │
    ┌────▼────┐ ┌────▼────┐ ┌───▼────┐
    │ Config  │ │   K8s   │ │ Docker │
    │  Files  │ │   API   │ │  API   │
    └─────────┘ └─────────┘ └────────┘
```

## Module Structure

- `cli/` - Command-line interface implementation
- `tui/` - Text-based UI implementation
- `config/` - Configuration management and schemas
- `services/` - Service management logic
- `core/` - Shared types and error handling

## Development Status

### Implemented
- ✅ Project structure and dependencies
- ✅ Configuration schema and management
- ✅ CLI argument parsing
- ✅ TUI interface with navigation
- ✅ Service state management
- ✅ Basic service operations (install, start, stop)
- ✅ Error handling and logging

### TODO (Post-MVP)
- [ ] Actual Kubernetes deployment integration
- [ ] Real-time log streaming
- [ ] Resource monitoring from Kubernetes metrics
- [ ] Service dependency resolution
- [ ] Configuration validation with schema
- [ ] Service health checks
- [ ] Auto-discovery of services from packages directory
- [ ] Multi-cluster support
- [ ] Plugin system

## Testing

The current implementation includes an offline mode that simulates service operations without requiring a Kubernetes cluster. This allows for testing the UI and workflows.

```bash
# Run in debug mode to see detailed logs
RUST_LOG=debug cargo run

# Run tests
cargo test
```

## License

[Your License Here]

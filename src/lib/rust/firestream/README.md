# Firestream CLI/TUI

A dual-interface tool providing both command-line arguments and an interactive text-based UI for managing data infrastructure services.

## Features

### MVP Implementation (✅ Completed)

- **Dual Interface**: Both CLI and TUI interfaces
- **Service Management**: Install, start, stop, and check status of services
- **Configuration Management**: Strongly-typed TOML configuration files
- **Service Discovery**: List available and installed services
- **Basic Monitoring**: View logs and resource usage
- **Offline Mode**: Works without Kubernetes connection for testing

## Installation

```bash
cd src/lib/rust/firestream
cargo build --release
```

## Usage

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

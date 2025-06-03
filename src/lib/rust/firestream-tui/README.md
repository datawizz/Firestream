# Firestream TUI

A terminal user interface for Firestream - a comprehensive Kubernetes resource management system that handles Airflow, Superset, Kafka, and templated applications.

## Architecture

The refactored TUI follows a modular architecture aligned with the OpenAPI specification:

```
firestream-tui/
├── src/
│   ├── main.rs          # Entry point
│   ├── app.rs           # Main application state and logic
│   ├── event.rs         # Event handling system
│   ├── ui.rs            # Main UI composition
│   ├── models/          # Data models matching OpenAPI spec
│   │   ├── mod.rs
│   │   ├── deployment.rs
│   │   ├── template.rs
│   │   ├── cluster.rs
│   │   ├── node.rs
│   │   ├── data.rs
│   │   ├── build.rs
│   │   └── secret.rs
│   ├── backend/         # API backend (with mock implementation)
│   │   ├── mod.rs
│   │   ├── api_client.rs    # Real API client (placeholder)
│   │   └── mock_client.rs   # Mock client for development
│   └── views/           # UI components
│       ├── mod.rs
│       ├── resources_pane.rs
│       ├── details_pane.rs
│       ├── logs_pane.rs
│       ├── help_view.rs
│       ├── command_palette.rs
│       └── search_view.rs
└── Cargo.toml
```

## Features

### Multi-Pane Navigation
- **Resources Pane** (left): Tree-based hierarchy of all resources
- **Details Pane** (top right): Detailed view of selected resource
- **Logs Pane** (bottom right): Log viewer for deployments and builds

### Resource Types
- **Deployments**: Running applications with status, replicas, and metrics
- **Templates**: Pre-configured application templates (PySpark, Python, Node.js)
- **Nodes**: Kubernetes nodes with GPU support
- **Data**: Delta tables, LakeFS branches, S3 buckets
- **Builds**: Container image builds with progress tracking
- **Secrets**: Kubernetes secrets management

### Navigation

#### Primary Navigation
- `j/k` or `↑/↓`: Navigate items in current pane
- `h/l` or `←/→`: Switch between panes
- `Space`: Expand/collapse tree nodes
- `Enter`: Select/activate item
- `Tab`: Move to next pane
- `Esc`: Back/cancel current operation

#### Quick Actions
- `/`: Global search overlay
- `:`: Command palette
- `?`: Context-sensitive help
- `n`: New (context-aware)
- `d`: Deploy/Delete (context-aware)
- `l`: View logs
- `s`: Scale/Search (context-aware)

### Command Palette
Access with `:` to run commands:
- `deploy <template>`: Deploy a new application
- `scale <deployment> <replicas>`: Scale deployment
- `logs <resource>`: View logs for a resource

### Search
Access with `/` to search across:
- Services
- Templates
- Kafka topics
- Documentation

## Usage

```bash
# Run the TUI
cargo run

# The TUI starts with a mock backend by default
# To use a real API backend, set the API_URL environment variable:
API_URL=http://localhost:8080/api/v1 cargo run
```

## Development

### Adding New Features

1. **New Resource Type**:
   - Add model in `src/models/`
   - Add backend methods in `src/backend/mod.rs`
   - Update mock implementation in `src/backend/mock_client.rs`
   - Add resource to tree in `src/app.rs`

2. **New View**:
   - Create view module in `src/views/`
   - Add to `View` enum in `src/views/mod.rs`
   - Handle rendering in `src/ui.rs`

3. **New Command**:
   - Add to command suggestions in `src/views/command_palette.rs`
   - Handle execution in `src/app.rs::execute_command()`

### Backend Integration

The TUI uses a trait-based backend system (`FirestreamBackend`) that allows easy switching between mock and real implementations:

```rust
// Use mock backend (default)
let backend = Arc::new(MockClient::new());

// Use real API backend
let backend = Arc::new(ApiClient::new("http://localhost:8080/api/v1".to_string()));
```

The `ApiClient` implementation is currently a placeholder. To implement:
1. Add HTTP client dependency (e.g., `reqwest`)
2. Implement actual HTTP calls in `src/backend/api_client.rs`
3. Handle authentication, error mapping, etc.

## Status Bar

The status bar shows:
- Firestream version
- Current environment (local-k3d, prod, etc.)
- Connection status
- Cluster uptime
- Resource usage (CPU/Memory)
- Context-sensitive key hints

## Future Enhancements

- [ ] Real-time log streaming
- [ ] Deployment configuration editor
- [ ] Resource creation wizards
- [ ] Metrics visualization
- [ ] Multi-cluster support
- [ ] Custom resource filtering
- [ ] Export/import configurations
- [ ] Integrated documentation viewer

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Firestream is a serverless data warehouse designed as "create-react-app for data engineering" - providing a fully local K3D cluster that can be deployed to production in GCP or AWS. It combines the Dataflow paradigm with Kubernetes infrastructure as code, emphasizing minimal data movement throughout its lifecycle.

### Core Concept
- Local development with K3D (Kubernetes in Docker) that mirrors production environments
- Infrastructure as Code via DevContainer with Docker-from-Docker pattern
- Supports real-time streaming data (webhooks, websockets, REST) with streaming effects
- Built 100% on open source (Bitnami charts, Apache projects, K8s ecosystem)

## Architecture

### Multi-Language Stack
1. **Rust** (Primary): TUI interface, API server, and infrastructure tooling
2. **Python**: ETL library, Airflow DAGs, data processing
3. **Scala**: Spark stateful streaming applications
4. **Node.js**: Middleware services, dashboard frontends
5. **Nix**: Reproducible build environment and dependency management

### Key Components

#### Rust Workspace (`src/lib/rust/`)
- `firestream-tui/`: Terminal UI for resource management (default workspace member)
- `firestream-api-server/`: Web API with PostgreSQL backend (multi-crate Gerust architecture)
  - `cli/`: DB migrations and code generation
  - `config/`: Environment-specific configuration management
  - `db/`: Database entities, migrations, access layer
  - `web/`: Web interface, controllers, auth middleware
  - `macros/`: Test macros
- `tools/`: Infrastructure management utilities
  - `k8s-manager`: Kubernetes resource operations
  - `helm-manager`: helm-CLI execution layer (helm/kubectl shell-out + values resolver; no embedded charts)
  - `firestream-charts`: chart registry — reads the flake-emitted index at `/opt/firestream/charts`
  - `docker-manager`: Container management
  - `templatizer-spark`: Spark application templating
  - `templatizer-puppeteer`: Node.js app templating
  - `templatizer-superset`: Superset dashboard templating
  - `filesystem-manager`: File system utilities

#### Python Libraries (`src/lib/python/`)
- `etl_lib/`: Core ETL utilities for Spark, S3, Delta Lake
- `configurator/`: Infrastructure configuration management
- `gen_proto/`: Generated Protocol Buffer bindings

#### Infrastructure
- **Containers**: Located in `src/containers/` (Bitnami fork)
- **Charts**: Located in `src/charts/` (Bitnami Helm charts fork)
- **Templates**: Project scaffolding in `src/templates/`
  - `standard_project/`: Basic project template
  - `standard_puppeteer/`: Node.js web app template
- **Deployment Packages**: Located in `src/deployment/packages/`

### DevContainer Architecture
The project uses a sophisticated DevContainer setup that:
- Runs K3D cluster on the host's Docker Engine
- Uses IP Tables to resolve Kubernetes internal DNS via CoreDNS
- Allows devcontainer processes to reach `service_name.namespace.svc.cluster.local`
- Essentially a seamless kubectl proxy that works across any host OS

## Development Commands

### Building and Running

```bash
# Build and run the TUI (default workspace member)
cargo run

# Build specific Rust component
cargo run -p firestream-api-server

# Build all Rust workspace members
cargo build --workspace

# Run Rust tests
cargo test --workspace

# Build Rust documentation
cargo doc --workspace --all-features
```

### Environment Setup

```bash
# Initialize development environment (runs bootstrap process)
make development

# Clean rebuild of environment
make development_clean

# Resume existing cluster (re-establish network tunnel)
make resume

# Build services and container registry
make build

# Run tests in CI/CD mode
make test
```

### Bootstrap Process
The `bootstrap.sh` script orchestrates deployment in different modes:
- `development`: Full K3D cluster + Helm charts + port forwarding
- `test`: CI/CD testing mode
- `build`: Build container images and artifacts
- `clean`: Create cluster without services
- `resume`: Reconnect to existing cluster
- `production`: Production deployment (TODO)

Modes execute in sequence:
1. K3D cluster setup (`docker/k3d/bootstrap.sh`)
2. DevContainer Python package installation (`bin/cicd_scripts/bootstrap_devcontainer.sh`)
3. Helm chart installation (`bin/cicd_scripts/helm_install.sh`)
4. Port forwarding setup (`bin/cicd_scripts/port_forward.sh`)

### Nix Environment

```bash
# Build the Nix flake (creates reproducible environment)
make nix-up

# Clean old Nix generations
make nix-fix
```

The Nix flake (`flake.nix`) provides:
- Fixed nixpkgs at release-24.11
- Rust toolchain via rustup
- Python 3.11 with key packages
- Scala, SBT, Java 11, Maven
- Node.js 18, gRPC tools
- Kafka (rdkafka), RocksDB
- LLVM/Clang toolchain
- Bitnami charts (fetched and cached at specific commit)

Environment activation: `source /etc/profile.d/nix-env.sh`

### Direnv Integration

The project uses [direnv](https://direnv.net) for automatic environment activation when entering the project directory.

```bash
# First-time setup (after installing direnv)
direnv allow

# Environment loads automatically on cd
cd /path/to/Firestream
# Output: "Firestream development environment loaded"
```

**How it works:**
- `.envrc` uses `use flake . --impure` to activate the Nix environment
- Loads `etc/.env.local` and `etc/.env.secrets` if they exist
- On macOS, sources `bin/darwin-env.sh` for Xcode/SDK configuration
- Local overrides can be placed in `.envrc.local` (git-ignored)

**Files:**
- `.envrc` - Main direnv configuration
- `.envrc.local.example` - Template for user-specific overrides
- `bin/darwin-env.sh` - macOS-specific environment setup

### Python Development

```bash
# Run Python tests
pytest

# Tests are located in src/lib/python/
# Test discovery: test_*.py files
```

### Docker and Kubernetes

```bash
# Build devcontainer
make build-devcontainer

# Build devcontainer without cache
make build-devcontainer-clean

# Delete all Docker resources (clean slate)
make docker-reset
```

## Working with Specific Components

### Firestream TUI
The TUI is the primary user interface, showing a tree-based resource hierarchy:
- **Navigation**: `j/k` or `↑/↓` (items), `h/l` or `←/→` (panes), `Space` (expand/collapse)
- **Quick Actions**: `/` (search), `:` (command palette), `?` (help), `n` (new), `d` (deploy/delete)
- **Panes**: Resources (left), Details (top right), Logs (bottom right)

The TUI uses a mock backend by default. Set `API_URL` environment variable for real API.

### API Server (Gerust Architecture)
The API server follows a multi-crate pattern:

```bash
# Run the API server
cargo run -p firestream-api-server

# Run database migrations
cargo db

# Generate scaffolding (entities, controllers, tests)
cargo generate

# Run API server tests
cargo test -p firestream-api-server
```

Configuration uses `.env` and `.env.test` files for development and test environments.

### Airflow Development

```bash
# Setup Airflow for local CLI testing
make airflow

# This runs:
# 1. DBT profile configuration
# 2. Local Airflow bootstrap
# 3. Test DAG execution
```

DAGs are located in deployment packages and examples directories.

## Data Stack Technologies

### Storage & Catalog
- **MinIO**: S3-compatible object storage
- **Delta Lake**: Table format
- **LakeFS**: Git-like data versioning
- **PostgreSQL**: Relational database (Bitnami chart)

### Processing
- **Apache Spark**: Distributed processing with Spark Operator
- **Kafka**: Message streaming (Bitnami chart)
- **Airflow**: Workflow orchestration

### Visualization
- **Superset**: BI dashboards
- **Plotly.js**: Real-time client-side dashboards

### Networking
- **Contour/Envoy**: Ingress controller
- **CoreDNS**: Internal DNS resolution

## Project Structure Patterns

### Rust Workspace Dependencies
The project uses workspace-level dependencies to ensure version consistency:
- Kubernetes: `k8s-openapi` with `v1_31` feature matching K3D version

### Multi-Crate Internal Dependencies
Rust tools reference each other via workspace paths:
```toml
templatizer-spark = { path = "src/lib/rust/tools/templatizer-spark" }
filesystem-manager = { path = "src/lib/rust/tools/filesystem-manager" }
```

### Python Editable Installs
Python packages in `src/lib/python/` are installed in editable mode during bootstrap for development.

## Key Architectural Decisions

### K3D over Kind
While the README mentions Kind, the project has standardized on K3D:
- Lighter weight than full K3s
- Better suited for local development
- Maintains compatibility with K8s 1.31 (matching `k8s-openapi` feature flag)

### Bitnami Charts Fork
The project maintains a fork of Bitnami charts in `src/charts/` and containers in `src/containers/`:
- Allows customization for Firestream-specific needs
- Ensures version stability
- Bitnami charts commit hash in Nix flake: `9bc801b4caa0b2fff6ae3392f6b417877a056965`

### Helm Chart Contract (Nix -> JSON -> Rust)
The flake is the source of truth for helm. Each chart at `src/charts/firestream/<name>/nix/` is a typed-options overlay using `lib.evalModules`. The shared option types live in `bin/nix/firestream/charts/lib/types/`; the per-chart evaluator is `bin/nix/firestream/charts/eval-chart.nix`.

**Manifest emission (schema v1):**
- Each chart emits a `chart-manifest.json` (chart name, version, repo, vendored subchart paths, default `values.yaml`, and a `_meta` block).
- `_meta.containerRefs` is the per-chart image-injection seam. Only `airflow` is wired today; the other 7 charts still render Bitnami's bundled images.
- Subchart vendoring helper: `bin/nix/firestream/charts/lib/vendor-subcharts.nix`. Values rendering helper: `bin/nix/firestream/charts/lib/to-values-yaml.nix`.

**Aggregate bundle:**
- `packages.firestream-charts-bundle` (see `nix/flake-modules/charts/`) is a symlink farm with a top-level `index.json` listing every chart and the `firestreamStacks.dev` composition.
- Default deploy path: `/opt/firestream/charts/`. Dev-shell sets `FIRESTREAM_CHARTS_DIR=$PWD/.firestream/charts` and best-effort builds the bundle on shell entry.
- 8 charts have typed overlays and are in `firestreamCharts` + `firestreamStacks.dev`: `airflow`, `postgresql`, `redis`, `kafka`, `spark`, `jupyterhub`, `superset`, `odoo`.

**Rust consumers:**
- `firestream-charts` crate (`src/lib/rust/firestream-charts/`): reader for the bundle (`index.json` + per-chart `chart-manifest.json`).
- `helm_lifecycle` module in the `firestream` crate consumes manifests via `chart_info_from_manifest`.
- `helm-manager` crate is strictly the helm/kubectl CLI execution layer (`helm_client`, `kubectl_client`, `values_resolver`, `deployment`); chart embedding (`embedded_charts.rs`, `include_dir`, the `HelmManager` aggregator) is gone.

**CLI:**
```bash
firestream helm deploy <chart>          # deploy a single chart from the bundle
firestream helm deploy-stack <stack>    # deploy a named stack (e.g. dev)
# Override the bundle location:
firestream helm deploy <chart> --charts-dir /custom/path
# or:  FIRESTREAM_CHARTS_DIR=/custom/path firestream helm deploy <chart>
```

**Gotchas:**
- Chart option modules MUST mirror the actual `values.yaml` hierarchy. Bitnami uses both flat (`odoo`) and hub-and-spoke (`superset`, `jupyterhub`) shapes — do not flatten/un-flatten arbitrarily.
- Random-secret normalisation for parity tests lives in `nix/flake-modules/charts/checks.nix` and covers every Bitnami chart family.

### Docker-from-Docker Pattern
Rather than Docker-in-Docker, the devcontainer binds to `/var/run/docker.sock`:
- Shares host's Docker Engine
- More efficient (no nested virtualization)
- Enables K3D cluster on host while developing in container

### Stateful Streaming Emphasis
The project specifically implements Wiener process demonstrations:
- Spark Structured Streaming generates monotonic sensor data
- Scala MapGroupsWithState merges N-1 and Nth records
- Python stateful streaming as performance comparison
- Node middleware bridges Kafka to WebSocket for browser clients

## Environment Variables

Key variables set by `bootstrap.sh`:
- `DEPLOYMENT_MODE`: One of `development`, `test`, `build`, `clean`, `resume`, `production`
- `MACHINE_ID`: Host machine identifier
- `GIT_COMMIT_HASH`: Current commit for container tagging
- `CPU_ARCHITECTURE`: Detected architecture (x86_64, aarch64)
- `TOTAL_CPU_RESOURCES`: Available CPU cores
- `TOTAL_MEMORY_RESOURCES`: Available memory in MB
- `HAS_NVIDIA_GPU`: Boolean for GPU detection
- `LOCAL_WORKSPACE_FOLDER`: Original host directory for Docker-from-Docker volume mounts

## CI/CD

GitHub Actions workflow (`.github/workflows/build.yaml`):
- Triggers on push to `staging`, `dev`, or `dependabot/**` branches
- Triggers on PRs to `main`
- Runs `make test` followed by `make build`
- 45-minute timeout
- Cleans up space by removing unnecessary GitHub Actions tools

## Development Tips

### DNS Resolution Testing
Test K8s internal DNS from devcontainer:
```bash
ping service_name.namespace.svc.cluster.local
```

### Resource Management
The devcontainer requires minimum:
- 4 CPUs
- 8GB memory
- 32GB storage

### GPU Support
If NVIDIA GPU detected, the system automatically installs container toolkit via `bin/host_scripts/nvidia-debian.sh`

### Port Forwarding
Default forwarded ports in devcontainer: 3000
Services are port-forwarded to localhost after deployment.

### Python Interpreter Path
In devcontainer: `/home/firestream/.python` (symlinked from Nix environment)

## Common Gotchas

1. **Running Outside Container**: `bootstrap.sh` detects if running outside Docker and auto-launches via `docker compose`
2. **Rust Analyzer**: Uses `--target-dir=target-ra` to avoid conflicts with main build
3. **CPU Architecture**: Build scripts auto-detect and adapt to x86_64 or ARM64
4. **Helm Charts**: Use the local fork in `src/charts/`, not upstream Bitnami. Chart deploys go through the flake-emitted bundle at `/opt/firestream/charts/` (see "Helm Chart Contract" above), not directly via `helm install` against `src/charts/firestream/<name>/`.
5. **Workspace Default Member**: `cargo run` at root runs the TUI, not the API server
6. **`helm-manager` is not a chart store**: it is a helm/kubectl CLI wrapper. Callers must pass `values.yaml` explicitly via `values_files` (no defaults sourced from embedded charts).

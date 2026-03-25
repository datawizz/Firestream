<!-- Firestream README -->

<p align="center">
  <img src="assets/images/firestream-banner.png" alt="Firestream" width="800" />
</p>

<p align="center">
  <strong>Deploy data infrastructure and generate production-ready apps from templates — all from a TUI.</strong>
</p>

<p align="center">
  <a href="https://www.apache.org/licenses/LICENSE-2.0.txt"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License" /></a>
  <a href="https://github.com/Cogent-Creation-Co/Firestream/actions"><img src="https://github.com/Cogent-Creation-Co/Firestream/actions/workflows/build.yaml/badge.svg" alt="CI" /></a>
</p>

---

Firestream is a terminal UI and CLI for deploying open-source data infrastructure and scaffolding production-ready applications from templates. It packages popular tools like Airflow, Kafka, Spark, and PostgreSQL as reproducible [Nix](https://nixos.wiki/wiki/Flakes)-built containers, and provides a code generation engine that can scaffold PySpark jobs, Scala streaming apps, web scrapers, Superset dashboards, and full multi-platform applications — configured interactively through the TUI or via YAML/JSON config files.

Works on Linux, macOS, and Windows (WSL). Only requires Docker.

<p align="center">
  <img src="assets/images/firestream-tui.gif" alt="Firestream TUI" width="800" />
</p>

---

## Table of Contents

- [Infrastructure Containers](#infrastructure-containers)
- [Application Templates](#application-templates)
- [Quick Start](#quick-start)
- [Template Generation](#template-generation)
- [Architecture](#architecture)
- [Rust Workspace](#rust-workspace)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Contributing](#contributing)

---

## Infrastructure Containers

Every container is built from a Nix flake for bit-for-bit reproducibility. A single `make` command builds and runs any service.

| Container | Version(s) | Category | Port |
|-----------|-----------|----------|------|
| [Apache Airflow](https://airflow.apache.org/) | 3.0.3 | Orchestration | `localhost:8090` |
| [Apache Kafka](https://kafka.apache.org/) | 4.0 (KRaft, no ZooKeeper) | Streaming | `localhost:9092` |
| [Apache Spark](https://spark.apache.org/) | 4.0.0 | Processing | `localhost:8080` |
| [Apache Superset](https://superset.apache.org/) | 4.x, 5.x | BI / Analytics | `localhost:8088` |
| [PostgreSQL](https://www.postgresql.org/) | 16, 17 | Database | `localhost:5432` |
| [Redis](https://redis.io/) | 7, 8 | Cache | `localhost:6379` |
| [JupyterHub](https://jupyter.org/hub) | 5.3.0 | Notebooks | `localhost:8000` |
| [Odoo](https://www.odoo.com/) | 18.0 | ERP | `localhost:8069` |
| [Supabase](https://supabase.com/) | latest | Backend-as-a-Service | `localhost:3000` |

---

## Application Templates

Firestream's templatizer generates complete, deployable project scaffolding from interactive prompts or config files. Templates are embedded in the binary — no network access needed.

### Code Generators (Tera-based)

These templates use the Tera engine to produce fully configured projects with Dockerfiles, Kubernetes manifests, tests, and CI config.

| Template | Output | Key Features |
|----------|--------|-------------|
| **PySpark Application** | Python Spark job with K8s Spark Operator manifests | S3, Delta Lake, Kafka integration; configurable resources; pytest suite |
| **Scala Spark Application** | Scala Spark job with SBT build and K8s manifests | Structured Streaming, MLlib, Delta Lake; Typesafe Config; ScalaTest |
| **Puppeteer DOM Scraper** | TypeScript web scraper with Puppeteer | Auth (form, API key, OAuth, cookie); retry with backoff; S3 upload; rate limiting |
| **Puppeteer Functional Scraper** | TypeScript scraper using Effect-TS | ADT-based DSL with interpreter pattern; same features as DOM scraper |
| **Superset Dashboard** | YAML export bundle for Superset import | Database connections, datasets, charts, dashboard layouts; ZIP or directory output |

### Project Boilerplates

Ready-to-use project structures you can clone and start building on.

| Template | Tech Stack | Description |
|----------|-----------|-------------|
| **Multi-Platform App** | Next.js 15, Tauri v2, SwiftUI, Nix | Full monorepo: web + desktop + iOS with shared TypeScript libs, Supabase auth, Stripe, Protocol Buffers |
| **Standard Puppeteer** | TypeScript, Puppeteer, Nix, Docker | Standalone production scraper with retry, S3, structured logging, integration tests |
| **Standard Project** | Python 3.11, Nix, pytest | Minimal Python project with modern tooling (uv, black, mypy, ruff) |

### Example Applications

Reference implementations demonstrating Firestream patterns.

| Example | Stack | Description |
|---------|-------|-------------|
| **Spark Structured Streaming** (x4) | Scala, Spark | Stateful streaming, MapGroupsWithState, windowed aggregations, Wiener process |
| **WebSocket Middleware** | Node.js, Kafka | Kafka-to-WebSocket proxy for real-time browser dashboards |
| **Web Crawler Server** | Python, Selenium | REST API for server-side HTML rendering |
| **Text-to-SQL** | Python | AI-powered SQL query generation (Spider dataset) |

### Template Status

| Template | Status |
|----------|--------|
| PySpark Application | Complete |
| Scala Spark Application | Complete |
| Puppeteer DOM Scraper | Complete |
| Puppeteer Functional Scraper | Complete |
| Multi-Platform App | Complete |
| Standard Puppeteer | Complete |
| Superset Dashboard | Generator works, templates need expansion |
| Standard Project | Functional, needs README |
| WebSocket Middleware | Core works, RBAC not yet implemented |
| Web Crawler Server | Skeleton |
| Text-to-SQL | Skeleton |
| Airflow Data Lake | Planned (empty) |

---

## Quick Start

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) (Docker Desktop on macOS/Windows, or Docker Engine on Linux)
- [GNU Make](https://www.gnu.org/software/make/)
- ~8 GB available RAM, ~32 GB disk space

### Run Infrastructure

```bash
git clone https://github.com/Cogent-Creation-Co/Firestream.git
cd Firestream

# Start any service -- auto-builds on first run
make airflow-start        # Airflow at http://localhost:8090
make kafka-start          # Kafka at localhost:9092
make postgres-17-start    # PostgreSQL 17 at localhost:5432
make redis-8-start        # Redis 8 at localhost:6379
make superset-start       # Superset at http://localhost:8088
make jupyterhub-start     # JupyterHub at http://localhost:8000
make odoo-start           # Odoo at http://localhost:8069
```

Every container follows the same `<service>-<action>` pattern:

```bash
make <service>-build         # Build the container image via Nix
make <service>-start         # Start (auto-builds if image is missing)
make <service>-stop          # Stop
make <service>-restart       # Restart
make <service>-logs          # Tail logs
make <service>-status        # Show container status
make <service>-credentials   # Print connection info and default credentials
make <service>-clean         # Stop, remove images, and delete volumes
```

```bash
make containers-build-all    # Build all containers
make containers-status       # Status of everything
make containers-clean-all    # Clean everything
```

### Generate an Application

```bash
# Build the Rust workspace (includes the templatizer CLI)
cargo build --workspace

# Generate a PySpark application
cargo run -p templatizer -- spark -n my-etl-job -l python -o ./output

# Generate a Scala Spark application
cargo run -p templatizer -- spark -n my-streaming-app -l scala -o ./output

# Generate a web scraper
cargo run -p templatizer -- puppeteer -n my-scraper -t dom_scraper -o ./output

# Generate a Superset dashboard export
cargo run -p templatizer -- superset -n my-dashboard -o ./output

# List all available templates
cargo run -p templatizer -- list --detailed

# Generate from a config file
cargo run -p templatizer -- spark -n my-app -l python -o ./output --config my-config.yaml
```

Or use the TUI for interactive configuration:

```bash
cargo run -p firestream-tui
```

---

## Template Generation

### How It Works

The templatizer is a Rust-based code generation engine that:

1. **Embeds all templates at compile time** — no file I/O or network needed at runtime
2. **Validates configuration** — required fields, type checking, dependency validation (e.g., S3 config required if S3 enabled)
3. **Renders via Tera** — with custom filters (`snake_case`, `camel_case`, `pascal_case`, `kebab_case`)
4. **Produces complete projects** — source code, tests, Dockerfiles, Kubernetes manifests, CI config, README

### Configuration

Templates can be configured via:

- **TUI** — interactive field editing with validation and computed values
- **CLI flags** — for simple overrides
- **YAML/JSON config files** — for reproducible generation

Example Spark config:

```yaml
app_name: "sensor-pipeline"
version: "1.0.0"
language: "python"
organization: "com.example"
s3_enabled: true
s3_bucket: "data-lake"
s3_region: "us-west-2"
kafka_enabled: true
kafka_bootstrap_servers: "localhost:9092"
kafka_topic: "sensor-readings"
driver_memory: "2g"
executor_memory: "4g"
num_executors: 3
```

Example Puppeteer config:

```yaml
site_name: "product_catalog"
site_url: "https://shop.example.com"
workflow_type: "dom-scraping"
auth_type: "form"
login_path: "/login"
s3_bucket: "scraped-data"
output_format: "parquet"
enable_retry: true
retry_attempts: 3
data_schema:
  fields:
    - name: "product_id"
      type: "string"
      required: true
    - name: "price"
      type: "number"
      required: true
dom_extraction:
  item_selector: ".product-card"
  fields:
    product_id:
      type: "attribute"
      selector: "[data-id]"
      attribute: "data-id"
    price:
      type: "text"
      selector: ".price"
```

---

## Architecture

<p align="center">
  <!-- IMAGE PLACEHOLDER: Architecture diagram showing the layers below -->
  <!-- <img src="assets/images/architecture.svg" alt="Firestream Architecture" width="700" /> -->
</p>

### Container Build System

Firestream's containers are organized in three layers:

**Container Definitions** — Each container in `src/containers/firestream/<name>/` has a `flake.nix` declaring exact versions and dependencies, a `module.nix` using Firestream factories for entrypoint and health checks, and a `docker-compose.yml` for local development.

**Nix Module System** — A shared library at `bin/nix/firestream/` provides composable shell function modules (logging, validation, filesystem, networking, service management, persistence, config, state, volumes) and container factories:

```
mkContainerModule          # Base container with entrypoint, env, and health checks
mkPythonContainerModule    # Python apps (uv2nix for deterministic pip)
mkJavaContainerModule      # JVM apps (Temurin JDK)
mkNodeContainerModule      # Node.js apps
```

**Build Orchestration** — `bin/build-container.sh` auto-detects the platform: native `nix build` on Linux, Docker-wrapped Nix on macOS/WSL with persistent store volumes for fast rebuilds.

### Template Engine

The templatizer crate provides three generators (Spark, Puppeteer, Superset), each with:
- A typed config struct with builder pattern and validation
- Tera template files embedded at compile time via `workspace-embed`
- CLI and TUI integration for interactive or scripted generation

### Why Nix?

Dockerfiles are non-deterministic — the same `Dockerfile` produces different images depending on when and where you build it. Nix flakes guarantee the same inputs always produce the same output, with every dependency pinned to an exact hash. On macOS, Nix runs inside Docker transparently.

---

## Rust Workspace

| Crate | Purpose |
|-------|---------|
| `firestream` | Main library aggregating all tools |
| `firestream-tui` | Terminal UI ([ratatui](https://ratatui.rs/)) for containers, clusters, and templates |
| `templatizer` | Code generation engine (Spark, Puppeteer, Superset) |
| `nix-container-builder` | Cross-platform Nix container builder |
| `docker-manager` | Docker API client via [bollard](https://github.com/fussybeaver/bollard) |
| `helm-manager` | Helm chart deployment with embedded charts |
| `k8s-manager` | K3D cluster lifecycle management |
| `firestream-vib` | Container verification and health-check validation |
| `workspace-embed` | Embeds workspace into portable standalone binaries |
| `wait-for-port` | TCP port readiness checker |
| `filesystem-manager` | File operations, glob matching, diff utilities |

### TUI

The TUI provides a three-pane interface:

- **Resources** (left) — tree view of containers, clusters, deployments, and templates
- **Details** (top right) — configuration, status, and metadata for the selected resource
- **Logs** (bottom right) — real-time output

Navigation: `j/k` or arrows to move, `Space` to expand/collapse, `Tab` to switch panes, `/` to search, `:` for command palette, `?` for help.

### Container Verification (VIB)

Every container is validated through the VIB pipeline:

- **Health checks** embedded via the Nix module system
- **Goss-based validation** for runtime testing
- **Vulnerability scanning** via [Trivy](https://trivy.dev/) and [Grype](https://github.com/anchore/grype)
- **SBOM generation** for supply-chain transparency

### Build Commands

```bash
cargo run                       # Run the main Firestream CLI
cargo run -p firestream-tui     # Run the TUI
cargo run -p templatizer -- list  # List available templates
cargo build --workspace         # Build all crates
cargo test --workspace          # Run all tests
```

---

## Development Environment

### Option A: Just Docker

Only requires Docker and Make. Nix builds run inside Docker automatically on macOS:

```bash
make airflow-start   # Works on macOS, Linux, WSL
```

### Option B: DevContainer (VS Code / GitHub Codespaces)

1. Open the repo in VS Code
2. Click "Reopen in Container" (or launch via GitHub Codespaces)
3. Docker-from-Docker pattern shares the host Docker Engine

Minimum: 4 CPUs, 8 GB RAM, 32 GB storage.

### Option C: Nix + direnv

```bash
cd Firestream
direnv allow    # Activates the Nix flake environment automatically
```

Full toolchain: Rust (Fenix), Python 3.11, Node.js 22, Scala 2.13, Java 11, and all infrastructure CLIs.

---

## Project Structure

```
Firestream/
├── bin/
│   ├── build-container.sh              # Cross-platform container build script
│   └── nix/firestream/                 # Nix module system
│       ├── lib/                        # Shell function libraries (log, fs, net, etc.)
│       ├── containers/                 # Container factories (base, python, java, node)
│       ├── apps/                       # Application factories
│       ├── rust/                       # mkRustPackage (Fenix + Crane)
│       ├── node/                       # mkNodePackage
│       ├── packages/                   # Built-from-source packages
│       └── tests/                      # Module tests
├── src/
│   ├── app/
│   │   ├── firestream-tui/             # Terminal UI (ratatui)
│   │   └── firestream-docs/            # Documentation site (fumadocs / Next.js)
│   ├── containers/firestream/          # Nix-built container definitions
│   │   ├── airflow/                    #   Apache Airflow 3.0.3
│   │   ├── kafka/                      #   Apache Kafka 4.0 (KRaft)
│   │   ├── spark/                      #   Apache Spark 4.0.0
│   │   ├── postgresql/                 #   PostgreSQL 16, 17
│   │   ├── redis/                      #   Redis 7, 8
│   │   ├── jupyterhub/                 #   JupyterHub 5.3.0
│   │   ├── odoo/                       #   Odoo 18.0
│   │   ├── superset/                   #   Apache Superset 4.x, 5.x
│   │   └── supabase/                   #   Supabase
│   ├── charts/firestream/              # Helm charts (Airflow, PostgreSQL, Redis)
│   ├── templates/                      # Application templates
│   │   ├── spark/                      #   PySpark & Scala Spark generators
│   │   ├── puppeteer/                  #   DOM & functional scraper generators
│   │   ├── superset/                   #   Superset dashboard generator
│   │   ├── multi_platform_app/         #   Next.js + Tauri + SwiftUI monorepo
│   │   ├── standard_puppeteer/         #   Standalone scraper boilerplate
│   │   ├── standard_project/           #   Python project boilerplate
│   │   ├── _app_template/              #   Generic Helm chart generator
│   │   └── _example_apps/              #   Reference implementations
│   └── lib/
│       ├── rust/                       # Rust workspace crates
│       └── python/                     # Python ETL library
├── docker/firestream/                  # DevContainer configuration
├── flake.nix                           # Root Nix flake
├── makefile                            # Primary build interface
└── Cargo.toml                          # Rust workspace root
```

---

## Kubernetes Deployment

Firestream containers can be deployed to Kubernetes via Helm charts:

```bash
ls src/charts/firestream/     # airflow/  postgresql/  redis/

helm install airflow src/charts/firestream/airflow/
```

The `k8s-manager` crate provides K3D cluster lifecycle management, and `helm-manager` handles chart deployment with environment-specific configuration.

---

## Documentation

```bash
make docs-dev     # Start local docs server at http://localhost:3001
make docs-build   # Build static documentation site
```

- [Getting Started](src/app/firestream-docs/content/docs/getting-started/)
- [Architecture](src/app/firestream-docs/content/docs/architecture/)
- [Guides](src/app/firestream-docs/content/docs/guides/)
- [Contributing](src/app/firestream-docs/content/docs/development/contributing.mdx)

---

## Contributing

Contributions are welcome! See the [Contributing Guide](src/app/firestream-docs/content/docs/development/contributing.mdx).

### Adding a New Container

1. Create `src/containers/firestream/<name>/flake.nix` using a container factory
2. Add a `module.nix` with entrypoint scripts and health checks
3. Add a `docker-compose.yml` for local development
4. Add `make` targets in `makefile` following the `<service>-<action>` pattern
5. Add VIB test specifications

### Adding a New Template

1. Create Tera templates in `src/templates/<type>/<name>/`
2. Add a typed config struct in the templatizer crate (`src/lib/rust/templatizer/`)
3. Wire up the generator with validation and Tera rendering
4. Add CLI subcommand and TUI integration
5. Include an `example_context.json` for documentation

### Running Tests

```bash
cargo test --workspace              # Rust tests
cd bin/nix/firestream && nix-build tests/  # Nix module tests
cargo run -p firestream-vib         # Container verification
```

---

## License

[Apache License 2.0](LICENSE)

---

<p align="center">
  Built by <a href="https://github.com/Cogent-Creation-Co">Cogent Creation Co.</a>
</p>

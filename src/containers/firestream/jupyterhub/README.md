# Firestream JupyterHub Container

Multi-user Jupyter notebook server built with the Firestream module system.

## Version

- **JupyterHub**: 5.3.0
- **Python**: 3.12
- **Base**: Debian Bookworm (slim)

## Quick Start

### Using Docker Compose (Development)

```bash
# Start JupyterHub with PostgreSQL
docker compose up -d

# Access JupyterHub
open http://localhost:8000

# Default credentials
# Username: admin
# Password: admin123

# Stop
docker compose down
```

### Using Nix (Production)

```bash
# Build Docker image
nix build .#dockerImage

# Load into Docker
docker load < result

# Run
docker run -d -p 8000:8000 -p 8081:8081 \
  -e JUPYTERHUB_USERNAME=admin \
  -e JUPYTERHUB_PASSWORD=secretpassword \
  firestream-jupyterhub:5.3.0
```

## Environment Variables

### Core Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `JUPYTERHUB_USERNAME` | `user` | Admin username |
| `JUPYTERHUB_PASSWORD` | `bitnami` | Admin password |
| `JUPYTERHUB_PROXY_PORT_NUMBER` | `8000` | Proxy port |
| `JUPYTERHUB_API_PORT_NUMBER` | `8081` | Hub API port |

### Database Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `JUPYTERHUB_DATABASE_TYPE` | `postgresql` | Database type (`postgresql` or `sqlite`) |
| `JUPYTERHUB_DATABASE_HOST` | `postgresql` | Database host |
| `JUPYTERHUB_DATABASE_PORT_NUMBER` | `5432` | Database port |
| `JUPYTERHUB_DATABASE_NAME` | `bitnami_jupyterhub` | Database name |
| `JUPYTERHUB_DATABASE_USER` | `bn_jupyterhub` | Database user |
| `JUPYTERHUB_DATABASE_PASSWORD` | - | Database password |

### Spawner Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `JUPYTERHUB_SPAWNER` | `localprocess` | Spawner type: `localprocess`, `simple`, `docker`, `kubernetes` |

### Authenticator Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `JUPYTERHUB_AUTHENTICATOR` | `pam` | Authenticator: `pam`, `dummy`, `oauth`, `ldap` |

## Docker Secrets

Supports Docker secrets via `_FILE` suffix:

```yaml
services:
  jupyterhub:
    secrets:
      - jupyterhub_password
    environment:
      - JUPYTERHUB_PASSWORD_FILE=/run/secrets/jupyterhub_password
```

## Volumes

| Path | Description |
|------|-------------|
| `/bitnami/jupyterhub` | Persistent data |
| `/docker-entrypoint-init.d` | Custom init scripts |

## Development

```bash
# Enter Nix development shell
nix develop

# Check flake
nix flake check

# Update Python dependencies
uv lock
```

## Build Targets

```bash
make help        # Show all targets
make build       # Build Docker image via Nix
make load        # Load built image into Docker
make up          # Start development environment
make down        # Stop development environment
make clean       # Clean build artifacts
make shell       # Open shell in running container
make logs        # View container logs
make test        # Test the container
make nix-check   # Check Nix flake
make lock        # Generate uv.lock and flake.lock
make dev-shell   # Enter Nix development shell
```

## Architecture

This container uses the Firestream module system:
- **module.nix**: Container module definition using `mkPythonContainerModule`
- **flake.nix**: Nix flake with uv2nix for Python dependencies
- **scripts/**: Lifecycle scripts (validate, config, init, secrets)

### Directory Structure

```
jupyterhub/
  docker-compose.yml    # Top-level development compose
  Makefile              # Build convenience targets
  README.md             # This file
  module.nix            # Firestream module definition
  flake.nix             # Nix flake for reproducible builds
  pyproject.toml        # Python dependencies
  uv.lock               # Locked Python dependencies
  scripts/              # Lifecycle scripts
    env-defaults.sh     # Default environment variables
    validate.sh         # Validation logic
    config.sh           # Configuration generation
    init.sh             # Initialization logic
    secrets.sh          # Secrets handling
  5/debian-12/          # Version-specific build
    Dockerfile          # Container build definition
    docker-compose.yml  # Version-specific compose
    rootfs/             # Container filesystem overlay
    prebuildfs/         # Pre-build filesystem overlay
```

## License

Apache-2.0

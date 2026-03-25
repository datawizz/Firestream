# Firestream Superset

Pure Nix-based Apache Superset container, replacing Bitnami Stacksmith dependencies with reproducible builds using `uv2nix` and the Firestream module system.

Supports multiple Superset versions (4.x and 5.x) with version-specific directories.

## Features

- **Multi-version support**: Superset 4.x and 5.x available side-by-side
- **Reproducible builds**: All dependencies managed via Nix flake and uv.lock
- **Multi-role support**: webserver, celery-worker, celery-beat, celery-flower, init
- **Docker secrets support**: All sensitive variables support the `_FILE` suffix pattern
- **Bitnami-compatible**: Preserves environment variable naming for easy migration

## Quick Start

### Building

```bash
# Build the default (v5) Docker image
nix build .#dockerImage

# Build a specific version
nix build .#superset-4
nix build .#superset-5

# Or from the project root
nix build .#superset          # default (v5)
nix build .#superset-4        # version 4.x
nix build .#superset-5        # version 5.x

# Load into Docker
docker load < result
```

### Running with Docker Compose

```bash
# For Superset 4.x
cd 4/
docker compose up -d

# For Superset 5.x
cd 5/
docker compose up -d

# Access Superset
open http://localhost:8088

# Default credentials: admin / admin
```

### Development Shell

```bash
# Enter development environment (default v5)
nix develop

# Version-specific shells
nix develop .#v4
nix develop .#v5

# Or use make in version directory
cd 5/
make dev
```

## Version Selection

| Version | Directory | Build Command | Image Tag |
|---------|-----------|---------------|-----------|
| 4.x | `4/` | `nix build .#superset-4` | `firestream-superset:4.1.1` |
| 5.x (default) | `5/` | `nix build .#superset-5` | `firestream-superset:5` |

The default version is 5.x. Use explicit version targets when you need a specific version.

## Environment Variables

### Core Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SUPERSET_ROLE` | `webserver` | Role: webserver, celery-worker, celery-beat, celery-flower, init |
| `SUPERSET_SECRET_KEY` | (auto-gen) | Flask SECRET_KEY (required, min 42 chars for production) |

### User Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SUPERSET_USERNAME` | `admin` | Admin username |
| `SUPERSET_PASSWORD` | `admin` | Admin password |
| `SUPERSET_EMAIL` | `admin@superset.local` | Admin email |
| `SUPERSET_FIRSTNAME` | `Superset` | Admin first name |
| `SUPERSET_LASTNAME` | `Admin` | Admin last name |

### Webserver Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SUPERSET_WEBSERVER_HOST` | `0.0.0.0` | Bind address |
| `SUPERSET_WEBSERVER_PORT_NUMBER` | `8088` | Listen port |
| `SUPERSET_WEBSERVER_WORKERS` | `4` | Gunicorn workers |
| `SUPERSET_WEBSERVER_WORKER_CLASS` | `gthread` | Worker class |
| `SUPERSET_WEBSERVER_THREADS` | `20` | Threads per worker |
| `SUPERSET_WEBSERVER_TIMEOUT` | `60` | Request timeout (seconds) |

### Database Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SUPERSET_DATABASE_DIALECT` | `postgresql` | Database type |
| `SUPERSET_DATABASE_HOST` | `postgresql` | Database host |
| `SUPERSET_DATABASE_PORT_NUMBER` | `5432` | Database port |
| `SUPERSET_DATABASE_NAME` | `superset` | Database name |
| `SUPERSET_DATABASE_USERNAME` | `superset` | Database user |
| `SUPERSET_DATABASE_PASSWORD` | `` | Database password |
| `SUPERSET_DATABASE_USE_SSL` | `no` | Use SSL connection |

### Redis Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_HOST` | `redis` | Redis host |
| `REDIS_PORT_NUMBER` | `6379` | Redis port |
| `REDIS_PASSWORD` | `` | Redis password |
| `REDIS_CELERY_DATABASE` | `0` | Redis DB for Celery broker |
| `REDIS_RESULTS_DATABASE` | `1` | Redis DB for results backend |
| `REDIS_CACHE_DATABASE` | `2` | Redis DB for cache |

### Init Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SUPERSET_LOAD_EXAMPLES` | `false` | Load example dashboards |
| `SUPERSET_SKIP_DATABASE_WAIT` | `no` | Skip waiting for database |
| `SUPERSET_IMPORT_DATASOURCES` | `` | Path to datasources YAML file |

### Celery Flower

| Variable | Default | Description |
|----------|---------|-------------|
| `FLOWER_BASIC_AUTH` | `` | Basic auth for Flower UI (format: user:password) |

## Docker Secrets

All sensitive variables support the `_FILE` suffix pattern:

```yaml
services:
  superset:
    environment:
      - SUPERSET_SECRET_KEY_FILE=/run/secrets/secret_key
      - SUPERSET_DATABASE_PASSWORD_FILE=/run/secrets/db_password
    secrets:
      - secret_key
      - db_password

secrets:
  secret_key:
    file: ./secrets/superset_secret_key.txt
  db_password:
    file: ./secrets/db_password.txt
```

## Multi-Role Deployment

Superset is deployed as multiple services, each running a specific role:

### Webserver (SUPERSET_ROLE=webserver)
Main application server. Handles HTTP requests and renders dashboards.

### Init (SUPERSET_ROLE=init)
One-time initialization. Runs database migrations, creates admin user, loads examples.

### Celery Worker (SUPERSET_ROLE=celery-worker)
Async query processing. Required for async queries and alerts/reports.

### Celery Beat (SUPERSET_ROLE=celery-beat)
Scheduled task runner. Required for scheduled reports and alerts.

### Celery Flower (SUPERSET_ROLE=celery-flower)
Celery monitoring UI. Optional, useful for debugging async tasks.

## File Structure

```
superset/
├── flake.nix              # Root flake for version selection
├── README.md              # This file
├── 4/                     # Superset 4.x
│   ├── flake.nix          # Nix flake with uv2nix integration
│   ├── flake.lock         # Locked Nix dependencies
│   ├── module.nix         # Firestream container module
│   ├── pyproject.toml     # Python dependencies (apache-superset==4.1.1)
│   ├── uv.lock            # Locked Python dependencies
│   ├── docker-compose.yml # Local development setup
│   ├── Makefile           # Build shortcuts
│   └── scripts/           # Runtime scripts
└── 5/                     # Superset 5.x
    ├── flake.nix          # Nix flake with uv2nix integration
    ├── module.nix         # Firestream container module
    ├── pyproject.toml     # Python dependencies (apache-superset>=5.0.0,<6.0.0)
    ├── uv.lock            # Locked Python dependencies (generate with uv lock)
    ├── docker-compose.yml # Local development setup
    ├── Makefile           # Build shortcuts
    └── scripts/           # Runtime scripts
```

## Building from Source

### Prerequisites

- Nix with flakes enabled
- Docker (for loading and running images)
- uv (for updating Python dependencies)

### Updating Dependencies

```bash
# In the version directory (e.g., 5/)
cd 5/

# Update uv.lock
uv lock

# Rebuild
nix build .#dockerImage
```

### Adding Database Drivers

Edit `pyproject.toml` in the appropriate version directory to add additional database drivers:

```toml
dependencies = [
    # ... existing deps ...
    "sqlalchemy-bigquery>=1.9.0",  # BigQuery
    "snowflake-sqlalchemy>=1.5.1", # Snowflake
]
```

Then regenerate the lock file:

```bash
uv lock
nix build .#dockerImage
```

## Version Differences

### Superset 4.x
- Python 3.11 required
- Marshmallow 3.x pinned (incompatible with 4.x)
- Stable, production-tested

### Superset 5.x
- Python 3.10-3.12 supported
- Uses `uv` package manager instead of `pip`
- Latest features and security updates

## Troubleshooting

### Container won't start

1. Check logs: `docker compose logs superset`
2. Verify database is accessible
3. Ensure SUPERSET_SECRET_KEY is set for production

### Async queries not working

1. Ensure celery-worker is running
2. Check Redis connectivity
3. Verify REDIS_HOST and REDIS_PORT_NUMBER

### Database migration errors

1. Stop all services: `docker compose down`
2. Delete database volume: `docker volume rm firestream-superset_postgresql_data`
3. Restart: `docker compose up -d`

### Version 5 uv.lock missing

The `5/uv.lock` file must be generated on Linux:

```bash
cd 5/
uv lock
```

## License

Apache-2.0

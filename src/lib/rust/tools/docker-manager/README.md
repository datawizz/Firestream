# Docker Manager

A comprehensive command-line tool for managing Docker containers, images, volumes, networks, and more.

## Features

- **Container Management**: List, create, start, stop, restart, remove containers, view logs, execute commands, and monitor stats
- **Image Management**: List, pull, push, tag, remove images, search Docker Hub, view history, and prune unused images
- **Volume Management**: Create, list, remove, inspect volumes, and prune unused volumes
- **Network Management**: Create, list, remove networks, connect/disconnect containers, and prune unused networks
- **System Operations**: View Docker info, version, disk usage, health checks, and prune all unused resources
- **Docker Compose**: Support for up, down, logs, ps, restart, and build operations
- **Image Building**: Build Docker images from Dockerfiles with progress tracking

## Installation

```bash
# From the workspace root
cargo install --path src/lib/rust/tools/docker-manager
```

## Usage

### Container Operations

```bash
# List all containers
docker-manager container list --all

# Start a container
docker-manager container start my-container

# Stop a container with timeout
docker-manager container stop my-container --timeout 30

# View container logs
docker-manager container logs my-container --follow --tail 100

# Execute command in container
docker-manager container exec my-container ls -la

# View container stats
docker-manager container stats my-container

# Create a new container
docker-manager container create --name my-app --image nginx:latest

# Remove a container
docker-manager container remove my-container --force
```

### Image Operations

```bash
# List images
docker-manager image list

# Pull an image
docker-manager image pull nginx --tag latest

# Remove an image
docker-manager image remove old-image --force

# Tag an image
docker-manager image tag my-image myrepo/my-image v1.0

# Search Docker Hub
docker-manager image search nginx --limit 10

# View image history
docker-manager image history nginx:latest

# Prune unused images
docker-manager image prune --all
```

### Volume Operations

```bash
# List volumes
docker-manager volume list

# Create a volume
docker-manager volume create my-data --driver local

# Remove a volume
docker-manager volume remove my-data --force

# Inspect a volume
docker-manager volume inspect my-data

# Prune unused volumes
docker-manager volume prune
```

### Network Operations

```bash
# List networks
docker-manager network list

# Create a network
docker-manager network create my-network --driver bridge --internal

# Connect container to network
docker-manager network connect my-container my-network

# Disconnect container from network
docker-manager network disconnect my-container my-network --force

# Remove a network
docker-manager network remove my-network

# Prune unused networks
docker-manager network prune
```

### System Operations

```bash
# Show Docker version
docker-manager system version

# Show Docker system info
docker-manager system info

# Show disk usage
docker-manager system df

# Check Docker health
docker-manager system health

# Prune all unused resources
docker-manager system prune
```

### Docker Compose Operations

```bash
# Start services
docker-manager compose up --dir ./my-project --detach

# Stop services
docker-manager compose down --dir ./my-project --volumes

# View logs
docker-manager compose logs --dir ./my-project --follow service1 service2

# List services
docker-manager compose ps --dir ./my-project

# Restart services
docker-manager compose restart --dir ./my-project service1

# Build services
docker-manager compose build --dir ./my-project --no-cache
```

### Building Images

```bash
# Build an image from current directory
docker-manager build --tag my-app:latest

# Build with custom Dockerfile
docker-manager build --tag my-app:latest --dockerfile Dockerfile.prod

# Build with build arguments
docker-manager build --tag my-app:latest --build-arg VERSION=1.0 --build-arg ENV=prod

# Build without cache
docker-manager build --tag my-app:latest --no-cache
```

## Global Options

- `--verbose` or `-v`: Enable verbose output for debugging

## Examples

### Complete workflow example

```bash
# Pull a base image
docker-manager image pull ubuntu:22.04

# Build a custom image
docker-manager build --tag my-app:latest

# Create and start a container
docker-manager container create --name my-app-instance --image my-app:latest
docker-manager container start my-app-instance

# Check container status
docker-manager container list

# View logs
docker-manager container logs my-app-instance --tail 50

# Execute a command
docker-manager container exec my-app-instance /bin/bash -c "echo 'Hello from container'"

# Stop and remove
docker-manager container stop my-app-instance
docker-manager container remove my-app-instance

# Clean up
docker-manager image remove my-app:latest
docker-manager system prune
```

## Integration with Firestream

This tool is part of the Firestream ecosystem and shares common patterns with other tools in the workspace:

- Uses `bollard` for Docker API access
- Follows the same error handling patterns
- Integrates with the workspace's logging infrastructure
- Compatible with the development container environment

## Development

To run tests:

```bash
cargo test -p docker-manager
```

To run with debug logging:

```bash
RUST_LOG=debug cargo run -p docker-manager -- --verbose container list
```

## License

This project is part of the Firestream platform and follows the same licensing terms.
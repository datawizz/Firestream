# Firestream JupyterHub - Docker Image Build
#
# Usage:
#   make build    - Build Docker image via Nix
#   make load     - Load built image into Docker
#   make up       - Start development environment
#   make down     - Stop development environment
#   make clean    - Clean build artifacts

.PHONY: build load up down clean shell logs test nix-check help lock dev-shell

VERSION := 5.3.0
IMAGE_NAME := firestream-jupyterhub
IMAGE_TAG := $(VERSION)-nix

help:
	@echo "Firestream JupyterHub Build Targets:"
	@echo "  make build     - Build Docker image via Nix"
	@echo "  make load      - Load built image into Docker"
	@echo "  make up        - Start development environment"
	@echo "  make down      - Stop development environment"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make shell     - Open shell in running container"
	@echo "  make logs      - View container logs"
	@echo "  make test      - Test the container"
	@echo "  make nix-check - Check Nix flake"
	@echo "  make lock      - Generate uv.lock and flake.lock"
	@echo "  make dev-shell - Enter Nix development shell"

# Build the Docker image using Nix
build:
	@echo "Building JupyterHub Docker image via Nix..."
	nix build .#dockerImage --print-build-logs
	@echo "Build complete: result -> $(IMAGE_NAME):$(IMAGE_TAG)"

# Load the image into Docker
load: build
	@echo "Loading image into Docker..."
	docker load < result
	@echo "Image loaded: $(IMAGE_NAME):$(IMAGE_TAG)"

# Start development environment
up:
	docker compose up -d
	@echo "JupyterHub available at http://localhost:8000"

# Stop development environment
down:
	docker compose down

# Clean up
clean:
	docker compose down -v --remove-orphans 2>/dev/null || true
	rm -f result
	@echo "Cleaned up"

# Open shell in running container
shell:
	docker compose exec jupyterhub bash

# View logs
logs:
	docker compose logs -f jupyterhub

# Run tests
test:
	@echo "Testing JupyterHub container..."
	docker compose up -d
	@sleep 10
	curl -s http://localhost:8000/hub/api | head -1 && echo "API responding" || echo "API not ready"
	docker compose down

# Check Nix flake
nix-check:
	nix flake check

# Generate lock files
lock:
	@echo "Generating uv.lock..."
	uv lock
	@echo "Generating flake.lock..."
	nix flake update
	@echo "Lock files generated"

# Development shell
dev-shell:
	nix develop

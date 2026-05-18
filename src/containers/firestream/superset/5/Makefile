# Firestream Superset - Build and Development Makefile
# Copyright Firestream. MIT License.

.PHONY: build load dev test up down logs clean lock help

# Default target
help:
	@echo "Firestream Superset - Nix-based container"
	@echo ""
	@echo "Build commands:"
	@echo "  make build      - Build the Docker image with Nix"
	@echo "  make load       - Load the built image into Docker"
	@echo "  make dev        - Enter the Nix development shell"
	@echo ""
	@echo "Development commands:"
	@echo "  make up         - Start all services with docker-compose"
	@echo "  make down       - Stop all services"
	@echo "  make logs       - Follow logs from all services"
	@echo "  make test       - Run integration tests"
	@echo ""
	@echo "Maintenance commands:"
	@echo "  make lock       - Regenerate uv.lock file"
	@echo "  make clean      - Remove build artifacts"
	@echo ""
	@echo "Access:"
	@echo "  Superset UI:    http://localhost:8088"
	@echo "  Flower UI:      http://localhost:5555"
	@echo "  Credentials:    admin / admin"

# Build the Docker image using Nix
build:
	@echo "Building Superset Docker image..."
	nix build .#dockerImage
	@echo "Build complete. Run 'make load' to load into Docker."

# Load the built image into Docker
load:
	@echo "Loading Superset image into Docker..."
	docker load < result
	@echo "Image loaded: firestream-superset:5"

# Enter development shell
dev:
	nix develop

# Start all services
up:
	docker compose up -d
	@echo ""
	@echo "Services starting..."
	@echo "  Superset UI:    http://localhost:8088"
	@echo "  Flower UI:      http://localhost:5555"
	@echo "  Credentials:    admin / admin"
	@echo ""
	@echo "Run 'make logs' to follow logs"

# Stop all services
down:
	docker compose down

# Follow logs
logs:
	docker compose logs -f

# Run integration tests
test: up
	@echo "Waiting for services to be ready..."
	@sleep 30
	@echo "Testing Superset health endpoint..."
	@curl -f http://localhost:8088/health || (echo "Health check failed" && exit 1)
	@echo ""
	@echo "Testing Flower health..."
	@curl -f http://localhost:5555/ || (echo "Flower check failed" && exit 1)
	@echo ""
	@echo "All tests passed!"
	$(MAKE) down

# Regenerate lock file
lock:
	uv lock
	@echo "uv.lock regenerated"

# Clean build artifacts
clean:
	rm -f result
	rm -rf .direnv
	@echo "Build artifacts cleaned"

# Full rebuild
rebuild: clean build load

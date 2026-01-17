# Firestream Redis - Docker Image Build

.PHONY: build build7 build8 push test clean help lock nix-build nix-build7 nix-build8 nix-load

IMAGE_NAME := firestream-redis
IMAGE_TAG := 7

help:
	@echo "Firestream Redis Build Targets:"
	@echo ""
	@echo "Nix Builds (Recommended):"
	@echo "  make nix-build   - Build Docker image via Nix (Redis 7)"
	@echo "  make nix-build7  - Build Docker image via Nix (Redis 7)"
	@echo "  make nix-build8  - Build Docker image via Nix (Redis 8)"
	@echo "  make nix-load    - Load Nix-built image into Docker"
	@echo ""
	@echo "Docker Compose:"
	@echo "  make test        - Test the Redis image"
	@echo "  make clean       - Remove images"
	@echo ""
	@echo "Development:"
	@echo "  make lock        - Generate flake.lock"

# Nix builds
nix-build: nix-build7

nix-build7:
	@echo "Building Redis 7 via Nix..."
	nix build .#dockerImage7
	@echo "Build complete: ./result"

nix-build8:
	@echo "Building Redis 8 via Nix..."
	nix build .#dockerImage8
	@echo "Build complete: ./result"

nix-load:
	@echo "Loading image into Docker..."
	docker load < result
	@echo "Image loaded"

# Testing
test:
	@echo "Testing Redis..."
	docker run --rm -e ALLOW_EMPTY_PASSWORD=yes $(IMAGE_NAME):$(IMAGE_TAG) redis-server --version

# Cleanup
clean:
	docker rmi $(IMAGE_NAME):7 2>/dev/null || true
	docker rmi $(IMAGE_NAME):8 2>/dev/null || true
	rm -f result

# Lock files
lock:
	@echo "Generating flake.lock..."
	nix flake update
	@echo "Lock file generated"

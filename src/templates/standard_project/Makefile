# Makefile for Standard Project Template

.PHONY: help build up down shell clean init update test

# Default target
help:
	@echo "Available commands:"
	@echo "  make init    - Initialize the project (generate docker-compose.devcontainer.yml)"
	@echo "  make build   - Build the Docker container"
	@echo "  make up      - Start the development container"
	@echo "  make down    - Stop the development container"
	@echo "  make shell   - Enter the development container"
	@echo "  make clean   - Clean up containers and images"
	@echo "  make update  - Update Nix flake dependencies"
	@echo "  make test    - Run a quick test of the development environment"

# Initialize project
init:
	@echo "Initializing project..."
	@chmod +x docker/standard_project/docker_preinit.sh
	@bash docker/standard_project/docker_preinit.sh

# Build container
devcontainer: init
	@echo "Building development container..."
	bash docker/standard_project/docker_preinit.sh && \
	docker-compose -f docker/standard_project/docker-compose.devcontainer.yml build

install:
	npm install -g @anthropic-ai/claude-code

# Start container
up: init
	@echo "Starting development container..."
	docker-compose -f docker/standard_project/docker-compose.devcontainer.yml up -d

# Stop container
down:
	@echo "Stopping development container..."
	docker-compose -f docker/standard_project/docker-compose.devcontainer.yml down

# Enter container shell
shell:
	@echo "Entering development container..."
	docker-compose -f docker/standard_project/docker-compose.devcontainer.yml exec devcontainer bash

# Clean up
clean:
	@echo "Cleaning up containers and images..."
	docker-compose -f docker/standard_project/docker-compose.devcontainer.yml down -v --rmi local

# Update Nix dependencies
update:
	@echo "Updating Nix flake..."
	nix flake update

# Test the development environment
test: up
	@echo "Testing development environment..."
	@echo "\nChecking Python:"
	@docker-compose -f docker/standard_project/docker-compose.yml exec -T devcontainer python --version
	@echo "\nChecking Node.js:"
	@docker-compose -f docker/standard_project/docker-compose.yml exec -T devcontainer node --version
	@echo "\nChecking Rust:"
	@docker-compose -f docker/standard_project/docker-compose.yml exec -T devcontainer rustc --version
	@echo "\nChecking Scala:"
	@docker-compose -f docker/standard_project/docker-compose.yml exec -T devcontainer scala -version
	@echo "\nChecking Docker-from-Docker:"
	@docker-compose -f docker/standard_project/docker-compose.yml exec -T devcontainer docker --version || echo "Docker not available (normal if socket not mounted)"
	@echo "\nEnvironment test complete!"

# Development shortcuts
.PHONY: python-shell rust-test node-install

python-shell:
	docker-compose -f docker/standard_project/docker-compose.devcontainer.yml exec devcontainer python

rust-test:
	docker-compose -f docker/standard_project/docker-compose.devcontainer.yml exec devcontainer cargo test

node-install:
	docker-compose -f docker/standard_project/docker-compose.devcontainer.yml exec devcontainer npm install

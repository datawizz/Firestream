# Firestream Odoo - Docker Image Build

.PHONY: build push clean test help lock

IMAGE_NAME := firestream-odoo
IMAGE_TAG := 18.0

help:
	@echo "Firestream Odoo Build Targets:"
	@echo "  make build  - Build Docker image"
	@echo "  make push   - Push to registry"
	@echo "  make test   - Test the image"
	@echo "  make clean  - Remove image"
	@echo "  make lock   - Generate uv.lock and flake.lock"

build:
	@echo "Building $(IMAGE_NAME):$(IMAGE_TAG)..."
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) -f 18/debian-12/Dockerfile ../../../..
	@echo "Build complete"

push:
	docker push $(IMAGE_NAME):$(IMAGE_TAG)

test:
	docker run --rm $(IMAGE_NAME):$(IMAGE_TAG) odoo --version

clean:
	docker rmi $(IMAGE_NAME):$(IMAGE_TAG) 2>/dev/null || true

lock:
	@echo "Generating uv.lock..."
	uv lock
	@echo "Generating flake.lock..."
	nix flake update
	@echo "Lock files generated"

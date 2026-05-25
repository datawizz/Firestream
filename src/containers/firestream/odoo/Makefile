# Firestream Odoo - Docker Image Build (Multi-Version)

.PHONY: build push clean test help lock lock-all

VERSION ?= 18
IMAGE_NAME := firestream-odoo
IMAGE_TAG := $(VERSION).0

help:
	@echo "Firestream Odoo Build Targets:"
	@echo "  make build          - Build Docker image (VERSION=18 by default)"
	@echo "  make build VERSION=17 - Build Odoo 17 image"
	@echo "  make push           - Push to registry"
	@echo "  make test           - Test the image"
	@echo "  make clean          - Remove image"
	@echo "  make lock           - Generate uv.lock for VERSION"
	@echo "  make lock-all       - Generate uv.lock for all versions"
	@echo ""
	@echo "Supported versions: 15, 16, 17, 18"

build:
	@echo "Building $(IMAGE_NAME):$(IMAGE_TAG) via Nix..."
	nix build .#odoo-$(VERSION) -L
	@echo "Build complete"

push:
	docker push $(IMAGE_NAME):$(IMAGE_TAG)

test:
	docker run --rm $(IMAGE_NAME):$(IMAGE_TAG) odoo --version

clean:
	docker rmi $(IMAGE_NAME):$(IMAGE_TAG) 2>/dev/null || true

lock:
	@echo "Generating uv.lock for Odoo $(VERSION)..."
	cd $(VERSION) && uv lock
	@echo "Lock file generated"

lock-all:
	@for v in 15 16 17 18; do \
		echo "Generating uv.lock for Odoo $$v..."; \
		(cd $$v && uv lock); \
	done
	@echo "All lock files generated"

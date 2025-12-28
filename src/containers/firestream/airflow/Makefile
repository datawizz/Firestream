# Firestream Airflow - Docker Image Build
# Copyright Firestream. Apache-2.0 License.

.PHONY: build build-nix build-docker push clean test help up down logs status

# Use centralized cross-platform build script
BUILD_SCRIPT := ../../../../bin/build-container.sh

IMAGE_NAME := firestream-airflow
IMAGE_TAG := 3.0.3
NIX_TAG := $(IMAGE_TAG)-nix

help:
	@echo "Firestream Airflow Build Targets:"
	@echo "  make build        - Build with Nix (cross-platform)"
	@echo "  make build-docker - Build with Dockerfile"
	@echo "  make up           - Start docker-compose services"
	@echo "  make down         - Stop docker-compose services"
	@echo "  make logs         - View service logs"
	@echo "  make status       - Show service status"
	@echo "  make test         - Test the image"
	@echo "  make clean        - Remove image"
	@echo "  make push         - Push to registry"

build:
	@$(BUILD_SCRIPT) airflow

build-nix:
	@$(BUILD_SCRIPT) airflow

build-docker:
	@$(BUILD_SCRIPT) airflow --dockerfile

up:
	docker compose up -d

down:
	docker compose down

logs:
	docker compose logs -f

status:
	docker compose ps

push:
	docker push $(IMAGE_NAME):$(IMAGE_TAG)

test:
	docker run --rm $(IMAGE_NAME):$(IMAGE_TAG) airflow version

clean:
	docker rmi $(IMAGE_NAME):$(IMAGE_TAG) $(IMAGE_NAME):$(NIX_TAG) 2>/dev/null || true

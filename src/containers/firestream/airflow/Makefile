# Firestream Airflow - Docker Image Build

.PHONY: build push clean test help

IMAGE_NAME := firestream-airflow
IMAGE_TAG := 3.0.3

help:
	@echo "Firestream Airflow Build Targets:"
	@echo "  make build  - Build Docker image"
	@echo "  make push   - Push to registry"
	@echo "  make test   - Test the image"
	@echo "  make clean  - Remove image"

build:
	@echo "Building $(IMAGE_NAME):$(IMAGE_TAG)..."
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) -f 3/debian-12/Dockerfile .
	@echo "Build complete"

push:
	docker push $(IMAGE_NAME):$(IMAGE_TAG)

test:
	docker run --rm $(IMAGE_NAME):$(IMAGE_TAG) airflow version

clean:
	docker rmi $(IMAGE_NAME):$(IMAGE_TAG) 2>/dev/null || true

#
# Makefile for a templated Helm-based app deployment on k3s
#

# -------------------------------------------------------------------
# VARIABLES
# -------------------------------------------------------------------

APP_NAME        := new_project_template
CHART_DIR       := helm/$(APP_NAME)
DOCKER_DIR      := docker/$(APP_NAME)
DOCKERFILE      := $(DOCKER_DIR)/Dockerfile
IMAGE_NAME      := local/$(APP_NAME):latest
NAMESPACE       := default
KUBECONTEXT     := k3s
RELEASE_NAME    := $(APP_NAME)
# If you'd like to tag by a version or commit hash, override this via CLI:
# make build IMAGE_TAG=<your_tag>
IMAGE_TAG       ?= latest

# -------------------------------------------------------------------
# COMMANDS
# -------------------------------------------------------------------

.PHONY: help build load deploy upgrade delete status logs test

## Show help
help:
	@echo "Available targets:"
	@echo "  make build      - Build docker image for the app"
	@echo "  make load       - Load the built image into local k3s (if required)"
	@echo "  make deploy     - Deploy or install the Helm chart into k3s"
	@echo "  make upgrade    - Upgrade the existing Helm release"
	@echo "  make delete     - Uninstall the Helm release from k3s"
	@echo "  make status     - Check the status of the Helm release"
	@echo "  make logs       - Tail logs from the pod(s) in the release"
	@echo "  make test       - Run helm tests for the release"
	@echo
	@echo "For advanced usage, set variables like IMAGE_TAG from the CLI, e.g.:"
	@echo "  make build IMAGE_TAG=v1.0.0"
	@echo

## Build docker image
build:
	@echo "Building Docker image: $(IMAGE_NAME)"
	@docker build -t $(IMAGE_NAME) -f $(DOCKERFILE) .

## Load the built image into local k3s if needed
load:
	@echo "Loading image into local k3s..."
	# Depending on your k3s setup, you may need to use 'ctr' or a local registry
	# If your k3s cluster is containerd-based:
	sudo k3s ctr images import $(shell docker save -o $(APP_NAME).tar $(IMAGE_NAME))
	rm -f $(APP_NAME).tar

## Deploy (install) the Helm release into k3s
deploy:
	@echo "Deploying $(APP_NAME) using helm..."
	helm upgrade --install \
		--kube-context $(KUBECONTEXT) \
		--namespace $(NAMESPACE) \
		--create-namespace \
		$(RELEASE_NAME) \
		$(CHART_DIR) \
		--set image.repository=$(IMAGE_NAME) \
		--set image.tag=$(IMAGE_TAG)

## Upgrade the existing Helm release
upgrade:
	@echo "Upgrading Helm release $(RELEASE_NAME)..."
	helm upgrade \
		--kube-context $(KUBECONTEXT) \
		--namespace $(NAMESPACE) \
		$(RELEASE_NAME) \
		$(CHART_DIR) \
		--set image.repository=$(IMAGE_NAME) \
		--set image.tag=$(IMAGE_TAG)

## Delete the Helm release (uninstall)
delete:
	@echo "Uninstalling Helm release $(RELEASE_NAME)..."
	helm uninstall \
		--kube-context $(KUBECONTEXT) \
		--namespace $(NAMESPACE) \
		$(RELEASE_NAME)

## Check the status of the Helm release
status:
	@echo "Checking Helm release status..."
	helm status \
		--kube-context $(KUBECONTEXT) \
		--namespace $(NAMESPACE) \
		$(RELEASE_NAME) || true
	@echo
	kubectl --context $(KUBECONTEXT) --namespace $(NAMESPACE) get pods

## Tail logs from all containers in the release
logs:
	@echo "Tailing logs for all pods in namespace $(NAMESPACE) with release $(RELEASE_NAME)..."
	@kubectl --context $(KUBECONTEXT) --namespace $(NAMESPACE) logs \
		-l app.kubernetes.io/instance=$(RELEASE_NAME) \
		-f --max-log-requests=10

## Run Helm tests for the release
test:
	@echo "Running Helm tests for release $(RELEASE_NAME)..."
	helm test \
		--kube-context $(KUBECONTEXT) \
		--namespace $(NAMESPACE) \
		$(RELEASE_NAME)

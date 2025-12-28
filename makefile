### Makefile ###

# A Pluripotent Makefile for Firestream.
# Designed to deploy in development, testing, and production environments.

BASEDIR=$(shell pwd)
PROJECT_NAME=firestream

# Devcontainer Development
DEVCONTAINER_COMPOSE := docker/firestream/docker-compose.devcontainer.yml
DEVCONTAINER_PREINIT := docker/firestream/docker_preinit.sh


development:
	# Deploy the development environment
	@bash -c 'cd $(BASEDIR) && bash bootstrap.sh development'


build-devcontainer:
	bash $(DEVCONTAINER_PREINIT)
	docker compose -f $(DEVCONTAINER_COMPOSE) build devcontainer

devcontainer-build:
	bash $(DEVCONTAINER_PREINIT)
	docker compose -f $(DEVCONTAINER_COMPOSE) build devcontainer

devcontainer-start:
	bash $(DEVCONTAINER_PREINIT)
	docker compose -f $(DEVCONTAINER_COMPOSE) up -d

devcontainer-stop:
	docker compose -f $(DEVCONTAINER_COMPOSE) down

nix-up:
	# Build the flake
	nix build .#container

nix-fix:
	# Delete old generations
	nix-collect-garbage -d

# # Run the setup script so paths are correctly set at login
# RUN /bin/bash /home/$HOST_USERNAME/result/bin/setup-container




build-devcontainer-clean:
	bash $(DEVCONTAINER_PREINIT)
	docker compose -f $(DEVCONTAINER_COMPOSE) build devcontainer --no-cache


development_clean:
	@bash -c 'cd $(BASEDIR) && bash bootstrap.sh clean'

# Reuse the existing cluster by re-establishing the network tunnel
resume:
	# Useful for resuming the container after a restart
	@bash -c 'cd $(BASEDIR) && bash bootstrap.sh resume'

# Test services
test:
	make development
	@bash -c 'cd $(BASEDIR) && bash bootstrap.sh test'

# Build services
build:
	# Establish local container registry through k3d
	@bash -c 'cd $(BASEDIR) && bash bootstrap.sh build'

# ==============================================================================
# Container Build System (Cross-Platform)
# ==============================================================================
# Centralized build script handles platform detection and Nix-in-Docker on macOS
BUILD_CONTAINER := bin/build-container.sh

# Generic container build targets
container-build-%:
	@$(BUILD_CONTAINER) $*

# ==============================================================================
# Airflow Development (Nix-based container)
# ==============================================================================
AIRFLOW_DIR := src/containers/firestream/airflow
AIRFLOW_COMPOSE := $(AIRFLOW_DIR)/docker-compose.yml

# Build the Airflow container (cross-platform)
airflow-build:
	@$(BUILD_CONTAINER) airflow

# Start Airflow services (auto-builds if image missing)
airflow-start:
	@if ! docker image inspect firestream-airflow:3.0.3 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) airflow-build; \
	fi
	docker compose -f $(AIRFLOW_COMPOSE) up -d
	@echo "Airflow is running at http://localhost:8090"

# Alias for airflow-start (build if needed + start)
airflow-up: airflow-start

# Stop Airflow services
airflow-stop:
	docker compose -f $(AIRFLOW_COMPOSE) down

# Restart Airflow services
airflow-restart: airflow-stop airflow-start

# View Airflow logs
airflow-logs:
	docker compose -f $(AIRFLOW_COMPOSE) logs -f

# View logs for specific service (usage: make airflow-logs-SERVICE)
airflow-logs-%:
	docker compose -f $(AIRFLOW_COMPOSE) logs -f $*

# Run Airflow CLI commands (usage: make airflow-cli CMD="dags list")
airflow-cli:
	docker compose -f $(AIRFLOW_COMPOSE) exec airflow airflow $(CMD)

# Clean up Airflow containers and images
airflow-clean: airflow-stop
	docker rmi firestream-airflow:3.0.3 firestream-airflow:3.0.3-nix 2>/dev/null || true
	docker compose -f $(AIRFLOW_COMPOSE) down -v

# Show Airflow service status
airflow-status:
	docker compose -f $(AIRFLOW_COMPOSE) ps

# ==============================================================================
# Odoo Development (Nix-based container)
# ==============================================================================
ODOO_COMPOSE := src/containers/firestream/odoo/docker-compose.yml

odoo-build:
	@$(BUILD_CONTAINER) odoo

odoo-start:
	@if ! docker image inspect firestream-odoo:18.0 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) odoo-build; \
	fi
	docker compose -f $(ODOO_COMPOSE) up -d

odoo-stop:
	docker compose -f $(ODOO_COMPOSE) down

# ==============================================================================
# PostgreSQL Development (Nix-based container)
# ==============================================================================
POSTGRES_COMPOSE := src/containers/firestream/postgresql/docker-compose.yml

postgres-build:
	@$(BUILD_CONTAINER) postgresql

postgres-build-%:
	@$(BUILD_CONTAINER) postgresql --version $*

postgres-16-start:
	@if ! docker image inspect firestream-postgresql:16 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) postgres-build-16; \
	fi
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) up -d

postgres-16-stop:
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) down

postgres-17-start:
	@if ! docker image inspect firestream-postgresql:17 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) postgres-build-17; \
	fi
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) up -d

postgres-17-stop:
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) down

build_devcontainer:
	@bash -c 'cd $(BASEDIR) && bash $(DEVCONTAINER_PREINIT)'
	@bash -c 'cd $(BASEDIR) && docker compose -f $(DEVCONTAINER_COMPOSE) build'

build_devcontainer_no_cache:
	@bash -c 'cd $(BASEDIR) && bash $(DEVCONTAINER_PREINIT)'
	@bash -c 'cd $(BASEDIR) && docker compose -f $(DEVCONTAINER_COMPOSE) build --no-cache'

# Start services
demo:
	export DEPLOYMENT_MODE="development" && make bootstrap

	# Create dispose of the tunnel and create a new one
	pkill ngrok
	bash /workspace/bin/commands/create_ngrok_reverse_proxy.sh

	# Apply Demo Data
	# POD_NAME=$(kubectl get pods --no-headers -o custom-columns=":metadata.name" | grep superset | head -n 1)
	# kubectl exec -it ${POD_NAME} -- superset load_examples
	# kubectl exec -it superset-69459c794f-r6j5k -- superset load_examples


	# Start Streaming Data Generation
	#nohup python /workspace/src/services/back_end/spark_applications/_boilerplate/metronome/src/main.py > /workspace/logs/metronome.log 2>&1 &



# Clean up
docker-reset:
	bash bin/commands/delete.sh


# Run Stress Tests
stress:
	make bootstrap

	bash /workspace/bin/commands/run_stress_tests.sh

load_plugins:
	bash /workspace/src/api/plugin_manager/bootstrap.sh



# setup:
# 	npm install --global yarn
# 	yarnpkg add react-native-web echarts echarts-for-react ws

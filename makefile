### Makefile ###

# A Pluripotent Makefile for Firestream.
# Designed to deploy in development, testing, and production environments.

.PHONY: help

help:
	@echo "Firestream Makefile"
	@echo ""
	@echo "Container Pattern: <container>-<action>"
	@echo "  Actions: build, start, stop, clean, logs, status, credentials"
	@echo ""
	@echo "Containers:"
	@echo "  airflow     - Apache Airflow workflow orchestration"
	@echo "  kafka       - Apache Kafka streaming platform (KRaft mode)"
	@echo "  odoo        - Odoo ERP"
	@echo "  postgres    - PostgreSQL (use postgres-16-* or postgres-17-*)"
	@echo "  redis       - Redis (use redis-7-* or redis-8-*)"
	@echo ""
	@echo "Examples:"
	@echo "  make airflow-start       # Start Airflow"
	@echo "  make kafka-start         # Start Kafka"
	@echo "  make postgres-17-start   # Start PostgreSQL 17"
	@echo "  make redis-7-start       # Start Redis 7"
	@echo "  make odoo-credentials    # Show Odoo login info"
	@echo ""
	@echo "Development:"
	@echo "  make development         # Deploy development environment"
	@echo "  make build-devcontainer  # Build devcontainer"
	@echo "  make resume              # Resume existing cluster"

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

# Show Airflow credentials
airflow-credentials:
	@echo "=== Airflow Credentials ==="
	@echo "URL:      http://localhost:8090"
	@echo "Username: airflow"
	@echo "Password: airflow"
	@echo ""
	@echo "Flower:   http://localhost:5555"

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

odoo-restart: odoo-stop odoo-start

odoo-logs:
	docker compose -f $(ODOO_COMPOSE) logs -f

odoo-status:
	docker compose -f $(ODOO_COMPOSE) ps

odoo-clean: odoo-stop
	docker rmi firestream-odoo:18.0 2>/dev/null || true
	docker compose -f $(ODOO_COMPOSE) down -v

odoo-credentials:
	@echo "=== Odoo Credentials ==="
	@echo "URL:      http://localhost:8069"
	@echo "Database: odoo"
	@echo "Email:    admin@example.com"
	@echo "Password: admin"

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

# Default version aliases (PostgreSQL 17)
postgres-start: postgres-17-start
postgres-stop: postgres-17-stop
postgres-clean: postgres-17-clean
postgres-logs: postgres-17-logs
postgres-status: postgres-17-status
postgres-credentials: postgres-17-credentials

# Version-specific logs
postgres-16-logs:
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) logs -f

postgres-17-logs:
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) logs -f

# Version-specific status
postgres-16-status:
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) ps

postgres-17-status:
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) ps

# Version-specific clean
postgres-16-clean: postgres-16-stop
	docker rmi firestream-postgresql:16 2>/dev/null || true
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) down -v

postgres-17-clean: postgres-17-stop
	docker rmi firestream-postgresql:17 2>/dev/null || true
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) down -v

# Version-specific credentials
postgres-16-credentials:
	@echo "=== PostgreSQL 16 Credentials ==="
	@echo "Host:     localhost"
	@echo "Port:     5432"
	@echo "Database: firestream"
	@echo "Username: firestream"
	@echo "Password: firestream"
	@echo ""
	@echo "Connection: psql -h localhost -U firestream -d firestream"

postgres-17-credentials:
	@echo "=== PostgreSQL 17 Credentials ==="
	@echo "Host:     localhost"
	@echo "Port:     5432"
	@echo "Database: firestream"
	@echo "Username: firestream"
	@echo "Password: firestream"
	@echo ""
	@echo "Connection: psql -h localhost -U firestream -d firestream"

# ==============================================================================
# Redis Development (Nix-based container)
# ==============================================================================
REDIS_COMPOSE := src/containers/firestream/redis/docker-compose.yml
REDIS_REPLICASET_COMPOSE := src/containers/firestream/redis/docker-compose-replicaset.yml

redis-build:
	@$(BUILD_CONTAINER) redis

redis-build-%:
	@$(BUILD_CONTAINER) redis --version $*

redis-7-start:
	@if ! docker image inspect firestream-redis:7-nix >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) redis-build-7; \
	fi
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) up -d
	@echo "Redis 7 is running at localhost:6379"

redis-7-stop:
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) down

redis-8-start:
	@if ! docker image inspect firestream-redis:8-nix >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) redis-build-8; \
	fi
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) up -d
	@echo "Redis 8 is running at localhost:6379"

redis-8-stop:
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) down

# Default version aliases (Redis 7)
redis-start: redis-7-start
redis-stop: redis-7-stop
redis-clean: redis-7-clean
redis-logs: redis-7-logs
redis-status: redis-7-status
redis-credentials: redis-7-credentials

# Version-specific logs
redis-7-logs:
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) logs -f

redis-8-logs:
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) logs -f

# Version-specific status
redis-7-status:
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) ps

redis-8-status:
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) ps

# Version-specific clean
redis-7-clean: redis-7-stop
	docker rmi firestream-redis:7-nix 2>/dev/null || true
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) down -v

redis-8-clean: redis-8-stop
	docker rmi firestream-redis:8-nix 2>/dev/null || true
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) down -v

# Version-specific credentials
redis-7-credentials:
	@echo "=== Redis 7 Credentials ==="
	@echo "Host:     localhost"
	@echo "Port:     6379"
	@echo "Password: (none - ALLOW_EMPTY_PASSWORD=yes for dev)"
	@echo ""
	@echo "Connection: redis-cli -h localhost -p 6379"

redis-8-credentials:
	@echo "=== Redis 8 Credentials ==="
	@echo "Host:     localhost"
	@echo "Port:     6379"
	@echo "Password: (none - ALLOW_EMPTY_PASSWORD=yes for dev)"
	@echo ""
	@echo "Connection: redis-cli -h localhost -p 6379"

# Redis replication mode
redis-replicaset-start:
	docker compose -f $(REDIS_REPLICASET_COMPOSE) up -d
	@echo "Redis replicaset started (primary: 6379, replica: 6380)"

redis-replicaset-stop:
	docker compose -f $(REDIS_REPLICASET_COMPOSE) down

redis-replicaset-logs:
	docker compose -f $(REDIS_REPLICASET_COMPOSE) logs -f

# ==============================================================================
# Kafka Development (Nix-based container, KRaft mode)
# ==============================================================================
KAFKA_DIR := src/containers/firestream/kafka
KAFKA_COMPOSE := $(KAFKA_DIR)/docker-compose.yml
KAFKA_CLUSTER_COMPOSE := $(KAFKA_DIR)/docker-compose-cluster.yml

# Build the Kafka container (cross-platform)
kafka-build:
	@$(BUILD_CONTAINER) kafka

# Start Kafka (auto-builds if image missing)
kafka-start:
	@if ! docker image inspect firestream-kafka:4.0 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) kafka-build; \
	fi
	docker compose -f $(KAFKA_COMPOSE) up -d
	@echo "Kafka is running at localhost:9092"

# Alias for kafka-start
kafka-up: kafka-start

# Stop Kafka
kafka-stop:
	docker compose -f $(KAFKA_COMPOSE) down

# Restart Kafka
kafka-restart: kafka-stop kafka-start

# View Kafka logs
kafka-logs:
	docker compose -f $(KAFKA_COMPOSE) logs -f

# Clean up Kafka containers and images
kafka-clean: kafka-stop
	docker rmi firestream-kafka:4.0 2>/dev/null || true
	docker compose -f $(KAFKA_COMPOSE) down -v

# Show Kafka service status
kafka-status:
	docker compose -f $(KAFKA_COMPOSE) ps

# Show Kafka connection info
kafka-credentials:
	@echo "=== Kafka Connection Info ==="
	@echo "Bootstrap Server: localhost:9092"
	@echo "Controller:       localhost:9093"
	@echo "Mode:             KRaft (no ZooKeeper)"
	@echo ""
	@echo "Test with:"
	@echo "  kafka-topics.sh --bootstrap-server localhost:9092 --list"
	@echo "  kafka-console-producer.sh --bootstrap-server localhost:9092 --topic test"
	@echo "  kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic test --from-beginning"

# Kafka cluster mode (3-node)
kafka-cluster-start:
	@if ! docker image inspect firestream-kafka:4.0 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) kafka-build; \
	fi
	docker compose -f $(KAFKA_CLUSTER_COMPOSE) up -d
	@echo "Kafka cluster started (brokers: 9092, 9094, 9096)"

kafka-cluster-stop:
	docker compose -f $(KAFKA_CLUSTER_COMPOSE) down

kafka-cluster-logs:
	docker compose -f $(KAFKA_CLUSTER_COMPOSE) logs -f

kafka-cluster-status:
	docker compose -f $(KAFKA_CLUSTER_COMPOSE) ps

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

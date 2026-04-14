### Makefile ###

# A Pluripotent Makefile for Firestream.
# Designed to deploy in development, testing, and production environments.

.PHONY: help

help:
	@echo "Firestream Makefile"
	@echo ""
	@echo "Container Pattern: <container>-<action>"
	@echo "  Actions: build, start, stop, restart, clean, logs, status, credentials"
	@echo ""
	@echo "=== Containers ==="
	@echo ""
	@echo "  Databases:"
	@echo "    postgres     - PostgreSQL (use postgres-16-* or postgres-17-*)"
	@echo "    redis        - Redis (use redis-7-* or redis-8-*)"
	@echo ""
	@echo "  Data Processing:"
	@echo "    spark        - Apache Spark cluster (master + worker)"
	@echo "    kafka        - Apache Kafka streaming (KRaft mode)"
	@echo ""
	@echo "  Orchestration:"
	@echo "    airflow      - Apache Airflow workflow orchestration"
	@echo ""
	@echo "  Applications:"
	@echo "    jupyterhub   - JupyterHub notebook server"
	@echo "    odoo         - Odoo ERP system"
	@echo "    superset     - Apache Superset BI dashboards"
	@echo "    supabase     - Supabase Backend-as-a-Service"
	@echo ""
	@echo "=== Examples ==="
	@echo ""
	@echo "  make airflow-start       # Start Airflow"
	@echo "  make kafka-start         # Start Kafka"
	@echo "  make postgres-17-start   # Start PostgreSQL 17"
	@echo "  make redis-7-start       # Start Redis 7"
	@echo "  make superset-start      # Start Superset"
	@echo "  make odoo-credentials    # Show Odoo login info"
	@echo ""
	@echo "=== Convenience ==="
	@echo ""
	@echo "  make containers-build-all   # Build all Nix containers"
	@echo "  make containers-status      # Status of all containers"
	@echo "  make containers-clean-all   # Clean all containers"
	@echo ""
	@echo "=== Development ==="
	@echo ""
	@echo "  make build-devcontainer  # Build devcontainer"
	@echo "  make devcontainer-start  # Start devcontainer"
	@echo "  make docker-reset        # Clean all Docker resources"
	@echo "  make nix-up              # Build Nix flake"
	@echo "  make nix-fix             # Garbage collect Nix"
	@echo "  make builder-cache-stats # Show Nix cache volume usage"
	@echo "  make builder-cache-clean # Remove Nix cache volumes"
	@echo ""
	@echo "=== SBOM / Fleet Manifest ==="
	@echo ""
	@echo "  make manifest            # Build fleet SBOM (CycloneDX + SPDX)"
	@echo "  make sbom-airflow        # Build individual container SBOM"
	@echo "  make manifest-validate   # Validate CycloneDX compliance"
	@echo "  make manifest-clean      # Clean manifest build artifacts"
	@echo ""
	@echo "=== Documentation ==="
	@echo ""
	@echo "  make docs-dev            # Start docs dev server (port 3001)"
	@echo "  make docs-build          # Build documentation site"
	@echo "  make docs-clean          # Clean docs build artifacts"

# ==============================================================================
# Core Variables
# ==============================================================================
BASEDIR=$(shell pwd)
PROJECT_NAME=firestream

# Devcontainer
DEVCONTAINER_COMPOSE := docker/firestream/docker-compose.devcontainer.yml
DEVCONTAINER_PREINIT := docker/firestream/docker_preinit.sh

# Documentation
DOCS_DIR := src/app/firestream-docs

# Container build script (handles cross-platform Nix builds)
BUILD_CONTAINER := bin/build-container.sh

# ==============================================================================
# Generic Container Build Pattern
# ==============================================================================
container-build-%:
	@$(BUILD_CONTAINER) $*

# ==============================================================================
# Shared Dependency Targets
# ==============================================================================
build-dep-redis:
	@if ! docker image inspect firestream-redis:7 >/dev/null 2>&1; then \
		echo "Building Redis..."; \
		$(BUILD_CONTAINER) redis; \
	else \
		echo "Redis image already exists"; \
	fi

build-dep-postgresql:
	@if ! docker image inspect firestream-postgresql:17 >/dev/null 2>&1; then \
		echo "Building PostgreSQL..."; \
		$(BUILD_CONTAINER) postgresql; \
	else \
		echo "PostgreSQL image already exists"; \
	fi

# ==============================================================================
# Airflow (Nix-based container)
# ==============================================================================
AIRFLOW_DIR := src/containers/firestream/airflow
AIRFLOW_COMPOSE := $(AIRFLOW_DIR)/docker-compose.yml

airflow-build-deps: build-dep-redis build-dep-postgresql
	@echo "==> Airflow dependencies ready"

airflow-build: airflow-build-deps
	@$(BUILD_CONTAINER) airflow

airflow-start:
	@if ! docker image inspect firestream-airflow:3.0.3 >/dev/null 2>&1; then \
		echo "Airflow image not found, building..."; \
		$(MAKE) airflow-build; \
	fi
	@if ! docker image inspect firestream-redis:7 >/dev/null 2>&1; then \
		echo "Redis image not found, building..."; \
		$(BUILD_CONTAINER) redis; \
	fi
	@if ! docker image inspect firestream-postgresql:17 >/dev/null 2>&1; then \
		echo "PostgreSQL image not found, building..."; \
		$(BUILD_CONTAINER) postgresql; \
	fi
	docker compose -f $(AIRFLOW_COMPOSE) up -d
	@echo "Airflow is running at http://localhost:8090"

airflow-up: airflow-start

airflow-stop:
	docker compose -f $(AIRFLOW_COMPOSE) down

airflow-restart: airflow-stop airflow-start

airflow-logs:
	docker compose -f $(AIRFLOW_COMPOSE) logs -f

airflow-logs-%:
	docker compose -f $(AIRFLOW_COMPOSE) logs -f $*

airflow-cli:
	docker compose -f $(AIRFLOW_COMPOSE) exec airflow airflow $(CMD)

airflow-clean: airflow-stop
	docker rmi firestream-airflow:3.0.3 firestream-airflow:3.0.3-nix 2>/dev/null || true
	docker compose -f $(AIRFLOW_COMPOSE) down -v

airflow-status:
	docker compose -f $(AIRFLOW_COMPOSE) ps

airflow-credentials:
	@echo "=== Airflow Credentials ==="
	@echo "URL:      http://localhost:8090"
	@echo "Username: airflow"
	@echo "Password: airflow"
	@echo ""
	@echo "Flower:   http://localhost:5555"

# ==============================================================================
# JupyterHub (Nix-based container)
# ==============================================================================
JUPYTERHUB_DIR := src/containers/firestream/jupyterhub
JUPYTERHUB_COMPOSE := $(JUPYTERHUB_DIR)/docker-compose.yml

jupyterhub-build:
	@$(BUILD_CONTAINER) jupyterhub

jupyterhub-start:
	@if ! docker image inspect firestream-jupyterhub:5.3.0-nix >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) jupyterhub-build; \
	fi
	docker compose -f $(JUPYTERHUB_COMPOSE) up -d
	@echo "JupyterHub is running at http://localhost:8000"

jupyterhub-up: jupyterhub-start

jupyterhub-stop:
	docker compose -f $(JUPYTERHUB_COMPOSE) down

jupyterhub-restart: jupyterhub-stop jupyterhub-start

jupyterhub-logs:
	docker compose -f $(JUPYTERHUB_COMPOSE) logs -f

jupyterhub-logs-hub:
	docker compose -f $(JUPYTERHUB_COMPOSE) logs -f jupyterhub

jupyterhub-clean: jupyterhub-stop
	docker rmi firestream-jupyterhub:5.3.0-nix 2>/dev/null || true
	docker compose -f $(JUPYTERHUB_COMPOSE) down -v

jupyterhub-status:
	docker compose -f $(JUPYTERHUB_COMPOSE) ps

jupyterhub-credentials:
	@echo "=== JupyterHub Credentials ==="
	@echo "URL:      http://localhost:8000"
	@echo "Username: admin"
	@echo "Password: admin123"

# ==============================================================================
# Kafka (Nix-based container, KRaft mode)
# ==============================================================================
KAFKA_DIR := src/containers/firestream/kafka
KAFKA_COMPOSE := $(KAFKA_DIR)/docker-compose.yml
KAFKA_CLUSTER_COMPOSE := $(KAFKA_DIR)/docker-compose-cluster.yml

kafka-build:
	@$(BUILD_CONTAINER) kafka

kafka-start:
	@if ! docker image inspect firestream-kafka:4.0 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) kafka-build; \
	fi
	docker compose -f $(KAFKA_COMPOSE) up -d
	@echo "Kafka is running at localhost:9092"

kafka-up: kafka-start

kafka-stop:
	docker compose -f $(KAFKA_COMPOSE) down

kafka-restart: kafka-stop kafka-start

kafka-logs:
	docker compose -f $(KAFKA_COMPOSE) logs -f

kafka-clean: kafka-stop
	docker rmi firestream-kafka:4.0 2>/dev/null || true
	docker compose -f $(KAFKA_COMPOSE) down -v

kafka-status:
	docker compose -f $(KAFKA_COMPOSE) ps

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

# ==============================================================================
# Odoo (Nix-based container)
# ==============================================================================
ODOO_DIR := src/containers/firestream/odoo
ODOO_COMPOSE := $(ODOO_DIR)/docker-compose.yml

odoo-build:
	@$(BUILD_CONTAINER) odoo

odoo-start:
	@if ! docker image inspect firestream-odoo:18.0 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) odoo-build; \
	fi
	docker compose -f $(ODOO_COMPOSE) up -d
	@echo "Odoo is running at http://localhost:8069"

odoo-up: odoo-start

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
# PostgreSQL (Nix-based container)
# ==============================================================================
POSTGRES_DIR := src/containers/firestream/postgresql
POSTGRES_COMPOSE := $(POSTGRES_DIR)/docker-compose.yml

postgres-build:
	@$(BUILD_CONTAINER) postgresql

postgres-build-%:
	@$(BUILD_CONTAINER) postgresql --version $*

# PostgreSQL 16
postgres-16-start:
	@if ! docker image inspect firestream-postgresql:16 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) postgres-build-16; \
	fi
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) up -d
	@echo "PostgreSQL 16 is running at localhost:5432"

postgres-16-stop:
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) down

postgres-16-restart: postgres-16-stop postgres-16-start

postgres-16-logs:
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) logs -f

postgres-16-status:
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) ps

postgres-16-clean: postgres-16-stop
	docker rmi firestream-postgresql:16 2>/dev/null || true
	PG_VERSION=16 docker compose -f $(POSTGRES_COMPOSE) down -v

postgres-16-credentials:
	@echo "=== PostgreSQL 16 Credentials ==="
	@echo "Host:     localhost"
	@echo "Port:     5432"
	@echo "Database: firestream"
	@echo "Username: firestream"
	@echo "Password: firestream"
	@echo ""
	@echo "Connection: psql -h localhost -U firestream -d firestream"

# PostgreSQL 17
postgres-17-start:
	@if ! docker image inspect firestream-postgresql:17 >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) postgres-build-17; \
	fi
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) up -d
	@echo "PostgreSQL 17 is running at localhost:5432"

postgres-17-stop:
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) down

postgres-17-restart: postgres-17-stop postgres-17-start

postgres-17-logs:
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) logs -f

postgres-17-status:
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) ps

postgres-17-clean: postgres-17-stop
	docker rmi firestream-postgresql:17 2>/dev/null || true
	PG_VERSION=17 docker compose -f $(POSTGRES_COMPOSE) down -v

postgres-17-credentials:
	@echo "=== PostgreSQL 17 Credentials ==="
	@echo "Host:     localhost"
	@echo "Port:     5432"
	@echo "Database: firestream"
	@echo "Username: firestream"
	@echo "Password: firestream"
	@echo ""
	@echo "Connection: psql -h localhost -U firestream -d firestream"

# Default PostgreSQL version aliases (17)
postgres-start: postgres-17-start
postgres-stop: postgres-17-stop
postgres-restart: postgres-17-restart
postgres-logs: postgres-17-logs
postgres-status: postgres-17-status
postgres-clean: postgres-17-clean
postgres-credentials: postgres-17-credentials

# ==============================================================================
# Redis (Nix-based container)
# ==============================================================================
REDIS_DIR := src/containers/firestream/redis
REDIS_COMPOSE := $(REDIS_DIR)/docker-compose.yml
REDIS_REPLICASET_COMPOSE := $(REDIS_DIR)/docker-compose-replicaset.yml

redis-build:
	@$(BUILD_CONTAINER) redis

redis-build-%:
	@$(BUILD_CONTAINER) redis --version $*

# Redis 7
redis-7-start:
	@if ! docker image inspect firestream-redis:7-nix >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) redis-build-7; \
	fi
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) up -d
	@echo "Redis 7 is running at localhost:6379"

redis-7-stop:
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) down

redis-7-restart: redis-7-stop redis-7-start

redis-7-logs:
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) logs -f

redis-7-status:
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) ps

redis-7-clean: redis-7-stop
	docker rmi firestream-redis:7-nix 2>/dev/null || true
	REDIS_VERSION=7 docker compose -f $(REDIS_COMPOSE) down -v

redis-7-credentials:
	@echo "=== Redis 7 Credentials ==="
	@echo "Host:     localhost"
	@echo "Port:     6379"
	@echo "Password: (none - ALLOW_EMPTY_PASSWORD=yes for dev)"
	@echo ""
	@echo "Connection: redis-cli -h localhost -p 6379"

# Redis 8
redis-8-start:
	@if ! docker image inspect firestream-redis:8-nix >/dev/null 2>&1; then \
		echo "Image not found, building..."; \
		$(MAKE) redis-build-8; \
	fi
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) up -d
	@echo "Redis 8 is running at localhost:6379"

redis-8-stop:
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) down

redis-8-restart: redis-8-stop redis-8-start

redis-8-logs:
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) logs -f

redis-8-status:
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) ps

redis-8-clean: redis-8-stop
	docker rmi firestream-redis:8-nix 2>/dev/null || true
	REDIS_VERSION=8 docker compose -f $(REDIS_COMPOSE) down -v

redis-8-credentials:
	@echo "=== Redis 8 Credentials ==="
	@echo "Host:     localhost"
	@echo "Port:     6379"
	@echo "Password: (none - ALLOW_EMPTY_PASSWORD=yes for dev)"
	@echo ""
	@echo "Connection: redis-cli -h localhost -p 6379"

# Default Redis version aliases (7)
redis-start: redis-7-start
redis-stop: redis-7-stop
redis-restart: redis-7-restart
redis-logs: redis-7-logs
redis-status: redis-7-status
redis-clean: redis-7-clean
redis-credentials: redis-7-credentials

# Redis replication mode
redis-replicaset-start:
	docker compose -f $(REDIS_REPLICASET_COMPOSE) up -d
	@echo "Redis replicaset started (primary: 6379, replica: 6380)"

redis-replicaset-stop:
	docker compose -f $(REDIS_REPLICASET_COMPOSE) down

redis-replicaset-logs:
	docker compose -f $(REDIS_REPLICASET_COMPOSE) logs -f

# ==============================================================================
# Spark (Nix-based container)
# ==============================================================================
SPARK_DIR := src/containers/firestream/spark
SPARK_COMPOSE := $(SPARK_DIR)/docker-compose.yml

spark-build:
	@$(BUILD_CONTAINER) spark

spark-start:
	docker compose -f $(SPARK_COMPOSE) up -d
	@echo "Spark Master UI: http://localhost:8080"

spark-up: spark-start

spark-stop:
	docker compose -f $(SPARK_COMPOSE) down

spark-restart: spark-stop spark-start

spark-logs:
	docker compose -f $(SPARK_COMPOSE) logs -f

spark-logs-master:
	docker compose -f $(SPARK_COMPOSE) logs -f spark

spark-logs-worker:
	docker compose -f $(SPARK_COMPOSE) logs -f spark-worker

spark-status:
	docker compose -f $(SPARK_COMPOSE) ps

spark-clean:
	docker compose -f $(SPARK_COMPOSE) down -v

spark-credentials:
	@echo "=== Spark Connection Info ==="
	@echo "Master UI:  http://localhost:8080"
	@echo "Master URL: spark://localhost:7077"
	@echo ""
	@echo "Submit job:"
	@echo "  spark-submit --master spark://localhost:7077 your-app.py"

# ==============================================================================
# Supabase (Official Docker Compose - no custom build)
# ==============================================================================
SUPABASE_DIR := src/containers/firestream/supabase/supabase/docker
SUPABASE_COMPOSE := $(SUPABASE_DIR)/docker-compose.yml

supabase-start:
	docker compose -f $(SUPABASE_COMPOSE) up -d
	@echo "Supabase Studio: http://localhost:3000"

supabase-up: supabase-start

supabase-stop:
	docker compose -f $(SUPABASE_COMPOSE) down

supabase-restart: supabase-stop supabase-start

supabase-logs:
	docker compose -f $(SUPABASE_COMPOSE) logs -f

supabase-status:
	docker compose -f $(SUPABASE_COMPOSE) ps

supabase-clean:
	docker compose -f $(SUPABASE_COMPOSE) down -v

supabase-credentials:
	@echo "=== Supabase Connection Info ==="
	@echo "Studio:     http://localhost:3000"
	@echo "API:        http://localhost:8000"
	@echo "Kong:       http://localhost:8000"
	@echo ""
	@echo "See .env file in $(SUPABASE_DIR) for API keys"

# ==============================================================================
# Superset (Nix-based container)
# ==============================================================================
SUPERSET_VERSION ?= 5
SUPERSET_DIR := src/containers/firestream/superset/$(SUPERSET_VERSION)
SUPERSET_COMPOSE := $(SUPERSET_DIR)/docker-compose.yml

ifeq ($(SUPERSET_VERSION),4)
  SUPERSET_IMAGE_TAG := 4.1.1
else
  SUPERSET_IMAGE_TAG := $(SUPERSET_VERSION)
endif

superset-build: superset-build-deps
	@$(BUILD_CONTAINER) superset

# Build superset dependencies (redis and postgresql)
superset-build-deps: build-dep-redis build-dep-postgresql
	@echo "==> Superset dependencies ready"

superset-start:
	@if ! docker image inspect firestream-superset:$(SUPERSET_IMAGE_TAG) >/dev/null 2>&1; then \
		echo "Superset image not found, building..."; \
		$(MAKE) superset-build; \
	fi
	@if ! docker image inspect firestream-redis:7 >/dev/null 2>&1; then \
		echo "Redis image not found, building..."; \
		$(BUILD_CONTAINER) redis; \
	fi
	@if ! docker image inspect firestream-postgresql:17 >/dev/null 2>&1; then \
		echo "PostgreSQL image not found, building..."; \
		$(BUILD_CONTAINER) postgresql; \
	fi
	docker compose -f $(SUPERSET_COMPOSE) up -d
	@echo "Superset is running at http://localhost:8088"

superset-up: superset-start

superset-stop:
	docker compose -f $(SUPERSET_COMPOSE) down

superset-restart: superset-stop superset-start

superset-logs:
	docker compose -f $(SUPERSET_COMPOSE) logs -f

superset-logs-web:
	docker compose -f $(SUPERSET_COMPOSE) logs -f superset

superset-logs-worker:
	docker compose -f $(SUPERSET_COMPOSE) logs -f superset-worker

superset-logs-flower:
	docker compose -f $(SUPERSET_COMPOSE) logs -f superset-flower

superset-status:
	docker compose -f $(SUPERSET_COMPOSE) ps

superset-clean: superset-stop
	docker rmi firestream-superset:$(SUPERSET_IMAGE_TAG) 2>/dev/null || true
	docker compose -f $(SUPERSET_COMPOSE) down -v

superset-credentials:
	@echo "=== Superset Credentials ==="
	@echo "URL:      http://localhost:8088"
	@echo "Username: admin"
	@echo "Password: admin"
	@echo ""
	@echo "Flower (task monitor): http://localhost:5555"

# ==============================================================================
# Convenience Targets
# ==============================================================================

# Build all Nix-based containers (excludes supabase which uses official compose)
# Uses batch mode for efficient serial builds with shared Nix cache
containers-build-all:
	@echo "Building all Firestream Nix containers (batch mode)..."
	@$(BUILD_CONTAINER) postgresql redis airflow kafka spark jupyterhub --version 15 odoo --version 16 odoo --version 17 odoo --version 18 odoo superset

# Clean all containers
containers-clean-all:
	@echo "Cleaning all Firestream containers..."
	-$(MAKE) airflow-clean
	-$(MAKE) jupyterhub-clean
	-$(MAKE) kafka-clean
	-$(MAKE) odoo-clean
	-$(MAKE) postgres-clean
	-$(MAKE) redis-clean
	-$(MAKE) spark-clean
	-$(MAKE) supabase-clean
	-$(MAKE) superset-clean
	@echo "All containers cleaned!"

# Status of all containers
containers-status:
	@echo "=== Airflow ===" && docker compose -f $(AIRFLOW_COMPOSE) ps 2>/dev/null || echo "Not running"
	@echo ""
	@echo "=== JupyterHub ===" && docker compose -f $(JUPYTERHUB_COMPOSE) ps 2>/dev/null || echo "Not running"
	@echo ""
	@echo "=== Kafka ===" && docker compose -f $(KAFKA_COMPOSE) ps 2>/dev/null || echo "Not running"
	@echo ""
	@echo "=== Odoo ===" && docker compose -f $(ODOO_COMPOSE) ps 2>/dev/null || echo "Not running"
	@echo ""
	@echo "=== PostgreSQL ===" && docker compose -f $(POSTGRES_COMPOSE) ps 2>/dev/null || echo "Not running"
	@echo ""
	@echo "=== Redis ===" && docker compose -f $(REDIS_COMPOSE) ps 2>/dev/null || echo "Not running"
	@echo ""
	@echo "=== Spark ===" && docker compose -f $(SPARK_COMPOSE) ps 2>/dev/null || echo "Not running"
	@echo ""
	@echo "=== Supabase ===" && docker compose -f $(SUPABASE_COMPOSE) ps 2>/dev/null || echo "Not running"
	@echo ""
	@echo "=== Superset ===" && docker compose -f $(SUPERSET_COMPOSE) ps 2>/dev/null || echo "Not running"

# ==============================================================================
# Devcontainer Management
# ==============================================================================

build-devcontainer:
	bash $(DEVCONTAINER_PREINIT)
	docker compose -f $(DEVCONTAINER_COMPOSE) build devcontainer

build-devcontainer-clean:
	bash $(DEVCONTAINER_PREINIT)
	docker compose -f $(DEVCONTAINER_COMPOSE) build devcontainer --no-cache

devcontainer-start:
	bash $(DEVCONTAINER_PREINIT)
	docker compose -f $(DEVCONTAINER_COMPOSE) up -d

devcontainer-stop:
	docker compose -f $(DEVCONTAINER_COMPOSE) down

# ==============================================================================
# Nix Environment
# ==============================================================================

nix-up:
	nix build .#container

nix-fix:
	nix-collect-garbage -d

# ==============================================================================
# Docker Utilities
# ==============================================================================

docker-reset:
	bash bin/commands/delete.sh

# ==============================================================================
# Builder Cache Management
# ==============================================================================

# Show Nix store cache usage (volume-based caching)
builder-cache-stats:
	@echo "=== Nix Store Cache Volumes ==="
	@docker volume ls --filter name=firestream-nix-store
	@echo ""
	@for vol in $$(docker volume ls -q --filter name=firestream-nix-store); do \
		echo "  $$vol"; \
	done

# Clean Nix store cache (reclaim disk space)
builder-cache-clean:
	@echo "Removing Nix store cache volumes..."
	-docker volume rm firestream-nix-store-amd64 2>/dev/null || true
	-docker volume rm firestream-nix-store-arm64 2>/dev/null || true
	@echo "Cache cleared. Next build will be slower (cold cache)."

# ==============================================================================
# SBOM / Fleet Manifest
# ==============================================================================

# Build script (handles git worktrees, Docker resources, etc.)
MANIFEST_SCRIPT := bin/build/manifest.sh

# Build fleet manifest in a Linux container (required for macOS)
# Produces: sbom-cyclonedx.json, sbom-spdx.json, manifest.json
manifest:
	@$(MANIFEST_SCRIPT)

# Build individual container SBOM in a Linux container
# Usage: make sbom-airflow, make sbom-spark, etc.
sbom-%:
	@$(MANIFEST_SCRIPT) $*

# Validate CycloneDX SBOM compliance
manifest-validate: manifest
	@echo "Validating CycloneDX SBOM..."
	@if command -v cyclonedx &>/dev/null; then \
		cyclonedx validate --input-file _build/manifest/sbom-cyclonedx.json; \
	else \
		echo "cyclonedx-cli not found. Install with: nix run nixpkgs#cyclonedx-cli"; \
		echo "Alternatively, check online at: https://cyclonedx.org/validator"; \
	fi

# Clean manifest build artifacts
manifest-clean:
	rm -rf _build/manifest _build/sbom

# ==============================================================================
# Documentation
# ==============================================================================

docs-dev:
	@echo "Starting documentation dev server..."
	@if [ ! -d "$(DOCS_DIR)/node_modules" ]; then \
		echo "Installing dependencies..."; \
		cd $(DOCS_DIR) && pnpm install; \
	fi
	cd $(DOCS_DIR) && pnpm dev

docs-build:
	@echo "Building documentation site..."
	@if [ ! -d "$(DOCS_DIR)/node_modules" ]; then \
		echo "Installing dependencies..."; \
		cd $(DOCS_DIR) && pnpm install; \
	fi
	cd $(DOCS_DIR) && pnpm build
	@echo "Documentation built successfully!"

docs-clean:
	@echo "Cleaning docs build artifacts..."
	rm -rf $(DOCS_DIR)/.next
	rm -rf $(DOCS_DIR)/out
	rm -rf $(DOCS_DIR)/.source
	@echo "Docs cleaned!"

docs-install:
	cd $(DOCS_DIR) && pnpm install

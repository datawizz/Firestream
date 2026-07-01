# Convenience targets for the Firestream Airflow docker-compose example.

PROJECT := firestream-airflow
# The generated compose file (resolved from the flake; tracks the pinned images).
COMPOSE = $(shell nix build .#airflow-compose --no-link --print-out-paths)/docker-compose.yml

.PHONY: help up down logs ps clean

help:
	@echo "Targets:"
	@echo "  up     Build + load the custom image (baked DAG), then 'docker compose up -d'"
	@echo "  down   Stop the stack (keeps the data volumes)"
	@echo "  logs   Tail logs for all services"
	@echo "  ps     Show service status"
	@echo "  clean  Stop the stack AND drop the data volumes (full reset)"

up:
	bash scripts/up.sh

down:
	docker compose -f "$(COMPOSE)" -p $(PROJECT) down

logs:
	docker compose -f "$(COMPOSE)" -p $(PROJECT) logs -f

ps:
	docker compose -f "$(COMPOSE)" -p $(PROJECT) ps

clean:
	docker compose -f "$(COMPOSE)" -p $(PROJECT) down -v

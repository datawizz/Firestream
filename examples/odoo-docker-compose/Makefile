# Convenience targets for the Firestream Odoo docker-compose example.

PROJECT := firestream-odoo
# The generated compose file (resolved from the flake; tracks the pinned image).
COMPOSE = $(shell nix build .#odoo-compose --no-link --print-out-paths)/docker-compose.yml

.PHONY: help up down logs ps clean

help:
	@echo "Targets:"
	@echo "  up     Build + load the custom image, then 'docker compose up -d' (scripts/up.sh)"
	@echo "  down   Stop the stack (keeps the postgresql_data volume)"
	@echo "  logs   Tail logs for all services"
	@echo "  ps     Show service status"
	@echo "  clean  Stop the stack AND drop the data volume (full reset)"

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

# Convenience targets for the Firestream Next.js docker-compose example.

PROJECT := firestream-nextjs
# The generated compose file (resolved from the flake; tracks the pinned images).
COMPOSE = $(shell nix build .#nextjs-compose --no-link --print-out-paths)/docker-compose.yml

.PHONY: help up down logs ps clean

help:
	@echo "Targets:"
	@echo "  up     Build + load the custom image (baked app), then 'docker compose up -d'"
	@echo "  down   Stop the stack (keeps the data volume)"
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

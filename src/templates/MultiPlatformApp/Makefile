# MultiPlatformApp Makefile
.PHONY: help
.DEFAULT_GOAL := help

# Variables
WEB_DIR := src/app/web
DESKTOP_DIR := src/app/desktop
IOS_DIR := src/app/ios
MOBILE_DIR := src/app/mobile
LIB_SHARED := src/lib/typescript/shared
LIB_PROTO := src/lib/typescript/proto-types
PKG_MGR := pnpm
TURBO := $(PKG_MGR) turbo
DOCKER_DIR := docker/multi_platform_app

# Platform detection
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Colors
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
NC := \033[0m

help: ## Show this help
	@echo "$(BLUE)MultiPlatformApp Commands$(NC)"
	@echo "======================================"
	@echo ""
	@echo "$(YELLOW)Setup:$(NC)"
	@echo "  make init               Initialize project"
	@echo "  make devcontainer       Build dev container"
	@echo "  make install            Install dependencies"
	@echo ""
	@echo "$(YELLOW)Development:$(NC)"
	@echo "  make run-web-dev        Start Next.js dev server"
	@echo "  make run-desktop-dev    Start Tauri dev"
	@echo "  make run-ios-dev        Build and run iOS app (Tauri, macOS only)"
	@echo "  make run-docs-dev       Start documentation site dev server"
	@echo ""
	@echo "$(YELLOW)Build:$(NC)"
	@echo "  make build              Build all packages"
	@echo "  make build-libs         Build shared libraries"
	@echo "  make build-web          Build web app"
	@echo "  make build-desktop      Build desktop app"
	@echo "  make build-ios          Build iOS app (macOS only)"
	@echo "  make build-docs         Build documentation site"
	@echo ""
	@echo "$(YELLOW)Database & Auth:$(NC)"
	@echo "  make setup-auth-env     Copy example env files for auth"
	@echo "  make check-auth-env     Verify auth environment configured"
	@echo "  make db-link            Link to remote Supabase project"
	@echo "  make db-migrate         Push migrations to Supabase"
	@echo "  make db-seed            Seed database with test data"
	@echo "  make db-generate-types  Generate TypeScript types from schema"
	@echo ""
	@echo "$(YELLOW)Stripe:$(NC)"
	@echo "  make stripe-listen      Forward webhooks to local server"
	@echo "  make stripe-fixtures    Load test products/prices"
	@echo ""
	@echo "$(YELLOW)Utilities:$(NC)"
	@echo "  make proto-generate     Generate types from proto files"
	@echo "  make clean              Clean build artifacts"
	@echo "  make lint               Lint all code"
	@echo "  make test               Run tests"
	@echo "  make typecheck          Type-check TypeScript"

# Initialization
init: ## Initialize project
	@echo "$(BLUE)Initializing project...$(NC)"
	@chmod +x $(DOCKER_DIR)/docker-preinit.sh 2>/dev/null || true
	@bash $(DOCKER_DIR)/docker-preinit.sh 2>/dev/null || echo "Docker setup not available"
	@echo "$(GREEN)Project initialized$(NC)"
	@echo ""
	@echo "$(YELLOW)Next steps:$(NC)"
	@echo "  1. Run 'make install' to install dependencies"
	@echo "  2. Run 'make run-web-dev' or 'make run-desktop-dev' to start development"

devcontainer: init ## Build development container
	@echo "$(BLUE)Building development container...$(NC)"
	docker compose -f $(DOCKER_DIR)/docker-compose.yml build
	@echo "$(GREEN)Devcontainer built$(NC)"

install: ## Install dependencies
	@echo "$(BLUE)Installing dependencies...$(NC)"
	$(PKG_MGR) install
	@echo "$(GREEN)Dependencies installed$(NC)"

# Development
run-web-dev: ## Start Next.js dev server
	@echo "$(BLUE)Starting web dev server...$(NC)"
	$(PKG_MGR) dev:web

run-desktop-dev: ## Start Tauri development
	@echo "$(BLUE)Starting desktop development...$(NC)"
	@-lsof -ti:1420 | xargs kill -9 2>/dev/null || true
	$(PKG_MGR) dev:desktop

run-docs-dev: ## Start documentation site dev server
	@echo "$(BLUE)Starting docs dev server...$(NC)"
	$(PKG_MGR) --filter docs dev

setup-ios-project: ## Generate Xcode project for iOS
ifeq ($(UNAME_S),Darwin)
	@echo "$(BLUE)Setting up iOS project...$(NC)"
	@which xcodegen > /dev/null 2>&1 || (echo "$(RED)XcodeGen not found. Enter the Nix dev shell first: nix develop$(NC)" && exit 1)
	@rm -rf $(IOS_DIR)/MultiPlatformApp.xcodeproj
	cd $(IOS_DIR) && xcodegen generate
	@echo "$(GREEN)iOS project generated$(NC)"
else
	@echo "$(RED)iOS development requires macOS$(NC)"
	@exit 1
endif

run-ios-sim: ## Launch iOS simulator
ifeq ($(UNAME_S),Darwin)
	@echo "$(BLUE)Launching iOS simulator...$(NC)"
	$(eval SIM_UDID := $(shell xcrun simctl list devices available -j | python3 -c "import sys,json; ver='$(IOS_SIMULATOR_VERSION)'; data=json.loads(sys.stdin.read()); devs=[(rt,d) for rt,ds in data['devices'].items() for d in ds if 'iPhone' in d['name']]; filtered=[d for rt,d in devs if ver and ver.replace('.','-') in rt.replace('.','-')] if ver else [d for _,d in devs]; print(filtered[0]['udid'] if filtered else '')"))
	@test -n "$(SIM_UDID)" || (echo "$(RED)No iPhone simulator found for iOS $(IOS_SIMULATOR_VERSION). Run 'xcrun simctl list runtimes' to see available versions.$(NC)" && exit 1)
	@xcrun simctl boot "$(SIM_UDID)" 2>/dev/null || true
	@open -a Simulator
	@xcrun simctl install "$(SIM_UDID)" \
		"$(IOS_DIR)/build/Build/Products/Debug-iphonesimulator/MultiPlatformApp.app"
	@xcrun simctl launch "$(SIM_UDID)" com.example.multiplatformapp
else
	@echo "$(RED)iOS development requires macOS$(NC)"
	@exit 1
endif

run-ios-dev: run-tauri-ios-dev ## Build and run iOS app (Tauri)

run-ios-native: setup-ios-project run-ios-build run-ios-sim ## Legacy: Build and run native iOS app
	@echo "$(GREEN)iOS app launched$(NC)"

# Tauri Mobile (iOS)
setup-tauri-ios: ## Initialize Tauri iOS project (run once)
ifeq ($(UNAME_S),Darwin)
	@echo "$(BLUE)Initializing Tauri iOS project...$(NC)"
	cd $(MOBILE_DIR) && $(PKG_MGR) tauri ios init --skip-targets-install
	@echo "$(YELLOW)Run 'make patch-tauri-ios' to apply custom build script$(NC)"
	@echo "$(GREEN)Tauri iOS project initialized$(NC)"
else
	@echo "$(RED)iOS development requires macOS$(NC)"
	@exit 1
endif

patch-tauri-ios: ## Patch Xcode project to use custom build script (run after setup-tauri-ios)
ifeq ($(UNAME_S),Darwin)
	@echo "$(BLUE)Patching Xcode project...$(NC)"
	@sed -i '' 's|pnpm tauri ios xcode-script|REPO_ROOT=\\"$$(cd \\"$$SRCROOT/../../../../../..\\" \&\& pwd)\\"\\nexec \\"$$REPO_ROOT/bin/tauri-ios-build.sh\\"|' \
		$(MOBILE_DIR)/src-tauri/gen/apple/multi_platform_app-mobile.xcodeproj/project.pbxproj
	@echo "$(GREEN)Xcode project patched$(NC)"
else
	@echo "$(RED)iOS development requires macOS$(NC)"
	@exit 1
endif

run-tauri-ios-dev: ## Start Tauri iOS development (simulator)
ifeq ($(UNAME_S),Darwin)
	@echo "$(BLUE)Starting Tauri iOS development...$(NC)"
	bash bin/tauri-ios-dev.sh
else
	@echo "$(RED)iOS development requires macOS$(NC)"
	@exit 1
endif

build-tauri-ios: ## Build Tauri iOS app
ifeq ($(UNAME_S),Darwin)
	@echo "$(BLUE)Building Tauri iOS app...$(NC)"
	cd $(MOBILE_DIR) && $(PKG_MGR) build
	xcodebuild -project $(MOBILE_DIR)/src-tauri/gen/apple/multi_platform_app-mobile.xcodeproj \
		-scheme multi_platform_app-mobile_iOS \
		-sdk iphonesimulator \
		-configuration release \
		-derivedDataPath $(MOBILE_DIR)/src-tauri/gen/apple/build
	@echo "$(GREEN)Tauri iOS build complete$(NC)"
else
	@echo "$(RED)iOS development requires macOS$(NC)"
	@exit 1
endif

run-ios-build: ## Build iOS app
ifeq ($(UNAME_S),Darwin)
	@echo "$(BLUE)Building iOS app...$(NC)"
	xcodebuild -project $(IOS_DIR)/MultiPlatformApp.xcodeproj \
		-scheme MultiPlatformApp \
		-sdk iphonesimulator \
		-configuration Debug \
		-derivedDataPath $(IOS_DIR)/build
	@echo "$(GREEN)iOS build complete$(NC)"
else
	@echo "$(RED)iOS development requires macOS$(NC)"
	@exit 1
endif

# Build
build: proto-generate ## Build all packages
	@echo "$(BLUE)Building all packages...$(NC)"
	$(TURBO) build
	@echo "$(GREEN)All packages built$(NC)"

build-libs: proto-generate ## Build shared libraries
	@echo "$(BLUE)Building shared libraries...$(NC)"
	$(TURBO) build --filter=@multi-platform-app/shared --filter=@multi-platform-app/proto-types --filter=@multi-platform-app/typescript-config --filter=@multi-platform-app/tailwind-config
	@echo "$(GREEN)Libraries built$(NC)"

build-web: build-libs ## Build web application
	@echo "$(BLUE)Building web application...$(NC)"
	$(TURBO) build --filter=web...
	@echo "$(GREEN)Web app built$(NC)"

build-desktop: build-libs ## Build desktop application
	@echo "$(BLUE)Building desktop application...$(NC)"
	cd $(DESKTOP_DIR) && $(PKG_MGR) tauri build
	@echo "$(GREEN)Desktop app built$(NC)"

build-ios: build-tauri-ios ## Build iOS application

build-docs: ## Build documentation site
	@echo "$(BLUE)Building documentation site...$(NC)"
	$(PKG_MGR) --filter docs build
	@echo "$(GREEN)Documentation built$(NC)"

# Proto generation
proto-generate: ## Generate types from proto files
	@echo "$(BLUE)Generating proto types...$(NC)"
	@bash scripts/proto-generate.sh
	@echo "$(GREEN)Proto types generated$(NC)"

# Utilities
clean: ## Clean build artifacts
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	$(TURBO) clean
	rm -rf $(LIB_SHARED)/dist
	rm -rf $(LIB_PROTO)/dist
	rm -rf $(LIB_PROTO)/src/generated
	rm -rf $(WEB_DIR)/.next
	rm -rf $(DESKTOP_DIR)/dist
	rm -rf $(DESKTOP_DIR)/src-tauri/target
	rm -rf $(MOBILE_DIR)/dist
	rm -rf $(MOBILE_DIR)/src-tauri/target
	rm -rf $(MOBILE_DIR)/src-tauri/gen/schemas
ifeq ($(UNAME_S),Darwin)
	rm -rf $(IOS_DIR)/build
	rm -rf $(IOS_DIR)/*.xcodeproj
endif
	@echo "$(GREEN)Cleaned$(NC)"

lint: ## Lint all code
	@echo "$(BLUE)Linting code...$(NC)"
	$(TURBO) lint
	@echo "$(GREEN)Linting complete$(NC)"

test: ## Run all tests
	@echo "$(BLUE)Running tests...$(NC)"
	$(TURBO) test
	@echo "$(GREEN)Tests complete$(NC)"

typecheck: ## Type-check TypeScript
	@echo "$(BLUE)Type-checking...$(NC)"
	$(TURBO) typecheck
	@echo "$(GREEN)Type-checking complete$(NC)"

# Platform info
info: ## Show platform information
	@echo "$(BLUE)Platform Information$(NC)"
	@echo "===================="
	@echo "OS: $(UNAME_S)"
	@echo "Architecture: $(UNAME_M)"
	@echo "Package Manager: $(PKG_MGR)"
	@echo "Node: $$(node --version 2>/dev/null || echo 'not installed')"
	@echo "pnpm: $$(pnpm --version 2>/dev/null || echo 'not installed')"
	@echo "Rust: $$(rustc --version 2>/dev/null || echo 'not installed')"
ifeq ($(UNAME_S),Darwin)
	@echo "Xcode: $$(xcodebuild -version 2>/dev/null | head -1 || echo 'not installed')"
	@echo "XcodeGen: $$(xcodegen --version 2>/dev/null || echo 'not installed')"
endif

# Open applications
open-web: ## Open web app in browser
	@open http://localhost:3000

open-desktop: ## Open desktop app
	@echo "$(BLUE)Desktop app runs via 'make run-desktop-dev'$(NC)"

# Docker shortcuts
docker-up: init ## Start Docker container
	@echo "$(BLUE)Starting development container...$(NC)"
	docker compose -f $(DOCKER_DIR)/docker-compose.yml up -d
	@echo "$(GREEN)Container started$(NC)"

docker-down: ## Stop Docker container
	@echo "$(BLUE)Stopping development container...$(NC)"
	docker compose -f $(DOCKER_DIR)/docker-compose.yml down
	@echo "$(GREEN)Container stopped$(NC)"

docker-shell: ## Enter Docker container shell
	@docker compose -f $(DOCKER_DIR)/docker-compose.yml exec devcontainer bash

# =====================
# Database & Auth
# =====================

LIB_AUTH := src/lib/typescript/auth

db-migrate: ## Push migrations to remote Supabase
	@echo "$(BLUE)Pushing database migrations...$(NC)"
	cd $(LIB_AUTH) && $(PKG_MGR) exec supabase db push
	@echo "$(GREEN)Migrations applied$(NC)"

db-seed: ## Seed database with test data
	@echo "$(BLUE)Seeding database...$(NC)"
	cd $(LIB_AUTH) && $(PKG_MGR) exec supabase db push
	@echo "$(GREEN)Database seeded$(NC)"

db-generate-types: ## Generate TypeScript types from Supabase schema
	@echo "$(BLUE)Generating database types...$(NC)"
	$(PKG_MGR) db:generate-types
	@echo "$(GREEN)Types generated$(NC)"

db-link: ## Link to remote Supabase project (interactive)
	@echo "$(BLUE)Linking to Supabase project...$(NC)"
	cd $(LIB_AUTH) && $(PKG_MGR) exec supabase link
	@echo "$(GREEN)Project linked$(NC)"

# =====================
# Stripe
# =====================

stripe-listen: ## Forward Stripe webhooks to local server
	@echo "$(BLUE)Starting Stripe webhook listener...$(NC)"
	@echo "$(YELLOW)Press Ctrl+C to stop$(NC)"
	stripe listen --forward-to localhost:3000/api/webhooks

stripe-fixtures: ## Load Stripe test products/prices
	@echo "$(BLUE)Loading Stripe fixtures...$(NC)"
	cd $(LIB_AUTH) && stripe fixtures stripe-fixtures.json
	@echo "$(GREEN)Stripe fixtures loaded$(NC)"

# =====================
# Auth Testing Setup
# =====================

check-auth-env: ## Verify auth environment is configured
	@echo "$(BLUE)Checking auth configuration...$(NC)"
	@test -f src/app/web/.env.local || (echo "$(RED)Missing src/app/web/.env.local$(NC)" && exit 1)
	@test -f src/app/desktop/.env || (echo "$(RED)Missing src/app/desktop/.env$(NC)" && exit 1)
	@test -f src/app/mobile/.env || (echo "$(RED)Missing src/app/mobile/.env$(NC)" && exit 1)
	@echo "$(GREEN)Environment files present$(NC)"

setup-auth-env: ## Copy example env files for auth testing
	@echo "$(BLUE)Setting up auth environment...$(NC)"
	@test -f src/app/web/.env.local || cp src/app/web/.env.example src/app/web/.env.local
	@test -f src/app/desktop/.env || cp src/app/desktop/.env.example src/app/desktop/.env
	@test -f src/app/mobile/.env || cp src/app/mobile/.env.example src/app/mobile/.env
	@echo "$(GREEN)Environment files created$(NC)"
	@echo ""
	@echo "$(YELLOW)Next steps:$(NC)"
	@echo "  1. Edit src/app/web/.env.local with your Supabase credentials"
	@echo "  2. Edit src/app/desktop/.env with your Supabase credentials"
	@echo "  3. Edit src/app/mobile/.env with your Supabase credentials"
	@echo "  4. Run 'make db-link' to link to your Supabase project"
	@echo "  5. Run 'make db-migrate' to apply migrations"
	@echo "  6. Run 'make stripe-fixtures' to load test products"

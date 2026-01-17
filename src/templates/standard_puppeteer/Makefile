# Makefile for example_shop scraper
# Generated for workflow: ecommerce_product_scraper

.PHONY: help build test run docker clean dev lint typecheck deploy-airflow

# Default target
help:
	@echo "example_shop Scraper - Available commands:"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Run in development mode with hot reload"
	@echo "  make build        - Build TypeScript to JavaScript"
	@echo "  make test         - Run test suite"
	@echo "  make lint         - Run linter"
	@echo "  make typecheck    - Run TypeScript type checker"
	@echo ""
	@echo "Production:"
	@echo "  make run          - Run the scraper (requires build)"
	@echo "  make docker       - Build Docker image"
	@echo "  make docker-run   - Run scraper in Docker"
	@echo ""
	@echo "Deployment:"
	@echo "  make deploy-airflow - Deploy to Airflow DAGs directory"
	@echo "  make package      - Create deployment package"
	@echo ""
	@echo "Testing:"
	@echo "  make test-demo    - Run tests against the-internet demo site"
	@echo "  make test-auth    - Test authentication workflows"
	@echo "  make test-forms   - Test form interactions"
	@echo "  make test-dynamic - Test dynamic content"
	@echo "  make test-debug   - Run tests with browser visible"
	@echo "  make test-report  - Generate test report"
	@echo "  make test-ci      - Run full CI test suite"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean        - Remove build artifacts"
	@echo "  make clean-test   - Remove test results"
	@echo "  make validate     - Validate workflow JSON"
	@echo "  make logs         - Tail application logs"

# Install dependencies
install:
	npm ci

# Development mode with hot reload
dev: install
	npm run dev

# Build TypeScript
build: install
	@echo "Building TypeScript..."
	npm run build
	@echo "Build complete"

# Run tests
test: install
	@echo "Running tests..."
	npm test

# Lint code
lint: install
	@echo "Running linter..."
	npm run lint

# Type check
typecheck: install
	@echo "Running type checker..."
	npm run typecheck

# Run the scraper (production mode)
run: build
	@echo "Running example_shop scraper..."
	NODE_ENV=production node dist/index.js

# Build Docker image
docker:
	@echo "Building Docker image..."
	docker build -t local/example_shop-scraper:1.0.0 .
	docker tag local/example_shop-scraper:1.0.0 \
		local/example_shop-scraper:latest
	@echo "Docker image built"

# Run in Docker
docker-run: docker
	@echo "Running in Docker..."
	docker run --rm \
		--env-file .env \
		-v $(PWD)/output:/app/output \
		local/example_shop-scraper:latest

# Push Docker image
docker-push: docker
	@echo "Pushing Docker image..."
	docker push local/example_shop-scraper:1.0.0
	docker push local/example_shop-scraper:latest

# Deploy to Airflow
AIRFLOW_DAGS_DIR ?= /opt/airflow/dags
deploy-airflow: build
	@echo "Deploying to Airflow..."
	@mkdir -p $(AIRFLOW_DAGS_DIR)/ecommerce_product_scraper
	@cp -r dist/* $(AIRFLOW_DAGS_DIR)/ecommerce_product_scraper/
	@cp workflow.json $(AIRFLOW_DAGS_DIR)/ecommerce_product_scraper/
	@echo "Deployed to $(AIRFLOW_DAGS_DIR)/ecommerce_product_scraper"

# Create deployment package
package: build
	@echo "Creating deployment package..."
	@mkdir -p packages
	@tar czf packages/example_shop-scraper-1.0.0.tar.gz \
		dist/ \
		node_modules/ \
		package*.json \
		workflow.json \
		Dockerfile \
		README.md
	@echo "Package created: packages/example_shop-scraper-1.0.0.tar.gz"

# Validate workflow JSON
validate:
	@echo "Validating workflow..."
	@node -e "const w = require('./workflow.json'); console.log('Workflow ID:', w.workflow_id); console.log('Effects:', w.effects.length);"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf dist/
	rm -rf coverage/
	rm -rf .parcel-cache/
	rm -rf packages/
	@echo "Clean complete"

# Tail logs (for development)
logs:
	@echo "Tailing logs..."
	@mkdir -p logs
	tail -f logs/*.log

# Environment setup check
check-env:
	@echo "Checking environment..."
	@node -e "console.log('Node.js:', process.version)"
	@echo "Puppeteer executable: $$PUPPETEER_EXECUTABLE_PATH"
	@echo "AWS Region: $$AWS_REGION"
	@echo "S3 Bucket: $$S3_BUCKET"
	@echo ""
	@if [ ! -f .env ]; then \
		echo "Warning: .env file not found. Copy .env.example to .env and configure."; \
	else \
		echo ".env file found"; \
	fi

# Run with local data (no S3 upload)
run-local: build
	@echo "Running with local data storage..."
	LOCAL_MODE=true node dist/index.js

# Performance profiling
profile: build
	@echo "Running with profiling..."
	node --inspect dist/index.js

# Generate TypeScript types
generate-types:
	@echo "Generating TypeScript types..."
	npm run generate-types

# Docker compose for local development with services
docker-compose-up:
	docker-compose up -d

docker-compose-down:
	docker-compose down

# Test targets for the-internet.herokuapp.com
test-demo: build
	@echo "Running tests against the-internet demo site..."
	npm run test:demo

test-auth: build
	@echo "Testing authentication workflows..."
	npm run test:demo -- --workflow=auth

test-forms: build
	@echo "Testing form interactions..."
	npm run test:demo -- --workflow=forms

test-dynamic: build
	@echo "Testing dynamic content..."
	npm run test:demo -- --workflow=dynamic

test-all-workflows: build
	@echo "Running all test workflows..."
	npm run test:demo -- --all

# Generate test report
test-report:
	@echo "Generating test report..."
	npm run test:report

# Run with headless=false for debugging
test-debug: build
	@echo "Running tests in debug mode (headless=false)..."
	HEADLESS=false npm run test:demo

# Run specific test with debug
test-debug-workflow: build
	@echo "Running specific test in debug mode..."
	@if [ -z "$(WORKFLOW)" ]; then \
		echo "Please specify WORKFLOW=<name>"; \
		exit 1; \
	fi
	HEADLESS=false npm run test:demo -- --workflow=$(WORKFLOW)

# Validate extracted data
validate-extraction:
	@echo "Validating extracted data..."
	npm run validate:data

# Performance tests
test-performance: build
	@echo "Running performance tests..."
	npm run test:performance

# Test with coverage
test-coverage: build
	@echo "Running tests with coverage..."
	npm run test:coverage

# Clean test results
clean-test:
	@echo "Cleaning test results..."
	rm -rf test-results/
	rm -rf coverage/
	rm -rf screenshots/

# Test environment setup
test-env-check:
	@echo "Checking test environment..."
	@echo "Target site: https://the-internet.herokuapp.com"
	@echo "Test output: ./test-results"
	@curl -s -o /dev/null -w "Site status: %{http_code}\n" https://the-internet.herokuapp.com/

# CI test runner
test-ci: clean-test test-env-check test-all-workflows test-report
	@echo "CI tests completed"
	@if [ -f test-results/test-summary.txt ]; then \
		cat test-results/test-summary.txt; \
	fi

# Shortcuts
d: dev
b: build
t: test
r: run
td: test-demo
tdb: test-debug
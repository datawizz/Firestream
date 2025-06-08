### Makefile ###

# A Pluripotent Makefile for Firestream.
# Designed to deploy in development, testing, and production environments.

BASEDIR=$(shell pwd)
PROJECT_NAME=firestream


development:
	# Deploy the development environment
	@bash -c 'cd $(BASEDIR) && bash bootstrap.sh development'


install:

	# splx command line tool
	cargo install sqlx-cli


cargo-reset:
	rm -rf ~/.cargo/registry/cache
	rm -rf ~/.cargo/registry/index

	# Set stable as default
	# Remove only the problematic toolchain
	rustup toolchain uninstall 1.85.0-x86_64-unknown-linux-gnu

	# Reinstall it
	rustup toolchain install 1.85.0

	# Set it as default again
	rustup default 1.85.0

reset-db:
	# Delete postgresql volume?
	docker-compose -f docker/firestream/docker-compose.devcontainer.yml down -v

db-migrations: install
	cd /workspace/src/lib/rust/firestream-api-server/db && \
	sqlx migrate run && \
	cargo sqlx prepare && \
	cargo build --package firestream-api-server-cli && \
	cargo run --package firestream-api-server-cli --bin db -- reset -e test && \
	cd /workspace/src/lib/rust/firestream-api-server && \
	cargo run --package firestream-api-server-cli --bin db -- seed -e test

devcontainer:
	bash docker/firestream/docker_preinit.sh
	docker compose -f docker/firestream/docker-compose.devcontainer.yml build devcontainer

nix-up:
	# Build the flake
	nix build .#container

nix-fix:
	# Delete old generations
	nix-collect-garbage -d

# # Run the setup script so paths are correctly set at login
# RUN /bin/bash /home/$HOST_USERNAME/result/bin/setup-container




build-devcontainer-clean:
	bash docker/docker_preinit.sh
	docker compose -f docker/docker-compose.devcontainer.yml build devcontainer --no-cache


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

airflow:
	# create the development environment
	# make development_clean
	# make development

	# Configure DBT Profile
	bash /workspace/src/plugins/airflow/config_dbt.sh

	# Deploy airflow locally for CLI testing
	bash /workspace/src/plugins/airflow/bootstrap_local.sh

	# Run a test
	python /workspace/src/plugins/airflow/dags/_examples/_template_dag_runnable.py

build_devcontainer:
	@bash -c 'cd $(BASEDIR) && bash docker/docker_preinit.sh'
	@bash -c 'cd $(BASEDIR) && docker compose -f docker/docker-compose.devcontainer.yml build'

build_devcontainer_no_cache:
	@bash -c 'cd $(BASEDIR) && bash docker/docker_preinit.sh'
	@bash -c 'cd $(BASEDIR) && docker compose -f docker/docker-compose.devcontainer.yml build --no-cache'

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

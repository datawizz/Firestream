### Makefile ###

# A Pluripotent Makefile for FireStream.
# Designed to deploy in development, testing, and production environments.

BASEDIR=$(shell pwd)
PROJECT_NAME=firestream



development:
	# Deploy the development environment
	@bash -c 'cd $(BASEDIR) && bash bootstrap.sh development'

build-devcontainer:
	bash docker/docker_preinit.sh
	docker compose -f docker/docker-compose.devcontainer.yml build devcontainer

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
	@bash -c 'cd $(BASEDIR) && bash bootstrap.sh clean'

	@bash -c 'cd $(BASEDIR) && bash bootstrap.sh build'


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
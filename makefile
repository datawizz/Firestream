### Makefile ###

# A Pluripotent Makefile for Fireworks.
# Designed to deploy in development, testing, and production environments.

BASEDIR=$(shell pwd)
PROJECT_NAME=fireworks

development:
	cd $(BASEDIR) && bash bootstrap.sh development


development_clean:
	cd $(BASEDIR) && bash bootstrap.sh clean

# Reuse the existing cluster by re-establishing the network tunnel
resume:
	# Useful for resuming the container after a restart
	export DEPLOYMENT_MODE="resume" && cd $(BASEDIR) && bash bootstrap.sh

# Test services
test:
	make bootstrap
	export DEPLOYMENT_MODE="test" && cd $(BASEDIR) && python -m pytest

# Build services
build:
	# The development environment contains a Local Registry
	make development_clean

	# Run the build scripts
	bash /workspace/docker/build.sh


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
boomboom:
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
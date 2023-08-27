### Makefile ###

# A Pluripotent Makefile for Fireworks.
# Designed to deploy in development, testing, and production environments.

BASEDIR=$(shell pwd)

devcontainer:
	export DEPLOYMENT_MODE="clean" && cd /workspace && bash bootstrap.sh


all:
	@echo "No argument suppllied. Making a standard build."
	export DEPLOYMENT_MODE="clean" && cd $(BASEDIR) && bash bootstrap.sh


config_host:
	cd /workspace && bash bin/commands/config_host.sh


bootstrap:
	cd /workspace && bash bootstrap.sh
	pip install -r /workspace/requirements.txt

resume:
	# Useful for resuming the container after a restart
	export DEPLOYMENT_MODE="resume" && cd $(BASEDIR) && bash bootstrap.sh

# Test services
test:
	export DEPLOYMENT_MODE="test" && cd $(BASEDIR) && bash bootstrap.sh
	export DEPLOYMENT_MODE="test" && cd $(BASEDIR) && python -m pytest

# Build services
build:
	bash /workspace/bin/cicd_scripts/build.sh


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



# # Stop services
# stop:
# 	# The commands to stop your services go here
# 	# They will depend on how your services are set up
# 	pkill -f your_python_script.py
# 	pkill -f your_bash_script.sh
# 	pkill -f your_java_program.jar

# Test services
test:
	make bootstrap

	bash /workspace/bin/commands/run_tests.sh

# Build services
build:
	bash /workspace/bin/cicd_scripts/build.sh

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
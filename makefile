### Makefile ###

# A Pluripotent Makefile for Fireworks.
# Designed to deploy in development, testing, cicd, and production environments.

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
	cd $(BASEDIR) && bash bootstrap.sh test

# Run Stress Tests
stress:
	cd $(BASEDIR) make test
	cd $(BASEDIR) && bash bin/commands/run_stress_tests.sh


# Build services
build:
	# Make the devcontainer
	make development_clean

	# Run the build scripts
	bash /workspace/bin/cicd_scripts/build.sh

deploy:
	# TODO
	echo "Deploying... TODO"

demo:
	echo "TODO: Demo Mode"
	# Wipe the cluster and boot the system
	# export DEPLOYMENT_MODE="clean" && cd $(BASEDIR) && bash bootstrap.sh
	# export DEPLOYMENT_MODE="development" && cd $(BASEDIR) && bash bootstrap.sh

	# Create dispose of the tunnel and create a new one
	# pkill ngrok
	# bash /workspace/bin/commands/create_ngrok_reverse_proxy.sh

	# Apply Demo Data
	# POD_NAME=$(kubectl get pods --no-headers -o custom-columns=":metadata.name" | grep superset | head -n 1)
	# kubectl exec -it ${POD_NAME} -- superset load_examples
	# kubectl exec -it superset-69459c794f-r6j5k -- superset load_examples


	# Start Streaming Data Generation
	#nohup python /workspace/src/services/back_end/spark_applications/_boilerplate/metronome/src/main.py > /workspace/logs/metronome.log 2>&1 &


# Clean up by deleting all the things
boomboom:
	# Delete the cluster first, then delete the project
	# k3d cluster delete fireworks
	bash bin/commands/delete.sh

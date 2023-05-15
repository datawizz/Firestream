### Makefile ###

# This makefile is used as a DevOps tool for the project to automate the following tasks:

bootstrap:
	cd /workspace && bash bootstrap.sh

development:
	cd /workspace && make bootstrap

# Start services
demo:

	make bootstrap

	# Start Streaming Data Generation
	python src/lib/python/etl_lib/tests/services/spark/pyspark_metronome/charts/main.py



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
purge_containers:
	bash /workspace/bin/commands/delete.sh

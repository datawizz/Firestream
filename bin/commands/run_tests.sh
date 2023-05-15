# bin/bash

# Run the projects tests for all compatible plugins and services

#   docker-compose -f docker/dev.docker-compose.yml build && docker-compose -f docker/dev.docker-compose.yml run devcontainer bash -c "/workspace/bootstrap.sh"

cd /workspace
pytest
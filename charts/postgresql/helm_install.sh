#!/bin/bash

# Check for required environment variables
# REQUIRED_VARS=("POSTGRES_USER" "POSTGRES_PASSWORD" "POSTGRES_DEFAULT_DB" "POSTGRES_PORT")
# MISSING_VARS=()

# for var in "${REQUIRED_VARS[@]}"; do
#   if [[ -z "${!var}" ]]; then
#     MISSING_VARS+=("$var")
#   fi
# done

# if [[ ${#MISSING_VARS[@]} -ne 0 ]]; then
#   echo "Error: The following required environment variables are missing:"
#   for missing_var in "${MISSING_VARS[@]}"; do
#     echo "  - $missing_var"
#   done
#   exit 1
# fi


###############################################################################
### Build Docker Image                                                     ###
###############################################################################

## Build each of the Dockerfiles in the project

# Build the dependencies and make it available on the local Docker registry
cd /workspace/src/services/persistence/postgresql/ && docker build --target deployment \
  -f /workspace/src/services/persistence/postgresql/Dockerfile \
  -t $CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/postgresql:$GIT_COMMIT_HASH \
  .
docker push $CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/postgresql:$GIT_COMMIT_HASH

# TODO build the rest of the images in the project?

# Proceed with helm install/upgrade command
helm install postgresql bitnami/postgresql -f /workspace/src/services/persistence/postgresql/chart/values.yaml \
  --set global.postgresql.auth.username="$POSTGRES_USER" \
  --set global.postgresql.auth.password="$POSTGRES_PASSWORD" \
  --set global.postgresql.auth.database="$POSTGRES_DEFAULT_DB" \
  --set global.postgresql.service.ports.postgresql="$POSTGRES_PORT" \
  --set image.registry=$CONTAINER_REGISTRY_URL" \
  --set image.repository="$CONTAINER_REGISTRY_URL/postgresql" \
  --set image.tag="$GIT_COMMIT_HASH"

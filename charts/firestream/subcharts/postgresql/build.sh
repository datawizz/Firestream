





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
### Websocket Middleware ###
cd /workspace/submodules/quadratichq/quadratic && docker build \
  --build-arg BASE_IMAGE=$BASE_IMAGE \
  -t k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/quadratic:$GIT_COMMIT_HASH \
  .

docker push k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/quadratic:$GIT_COMMIT_HASH




PLUGIN_DIR="/workspace/src/api/"

### Variables ###

SERVICE_NAME="websocket-middleware"
CONTAINER_REGISTRY_PREFIX="k3d-firestream."
CONTAINER_REGISTRY_URL="localhost"
CONTAINER_REGISTRY_URL_FULL=$CONTAINER_REGISTRY_PREFIX$CONTAINER_REGISTRY_URL

cd $PLUGIN_DIR$SERVICE_NAME


### Build ###

IMAGE_TAG=$CONTAINER_REGISTRY_URL_FULL:$CONTAINER_REGISTRY_PORT/$SERVICE_NAME:$GIT_COMMIT_HASH

docker build \
  --build-arg SERVICE_NAME=$SERVICE_NAME \
  -t $IMAGE_TAG \
  .

docker push $IMAGE_TAG


cd $PLUGIN_DIR$SERVICE_NAME/chart/websocket-middleware



helm upgrade --install $SERVICE_NAME . \
    --set image.repository=$CONTAINER_REGISTRY_URL_FULL:$CONTAINER_REGISTRY_PORT/$SERVICE_NAME \
    --set image.tag=$GIT_COMMIT_HASH \
    --set image.pullPolicy=Always \
    -f /workspace/src/api/websocket-middleware/manifest.a.yaml


kubectl apply -f /workspace/src/api/websocket-middleware/ingressRoute.yaml
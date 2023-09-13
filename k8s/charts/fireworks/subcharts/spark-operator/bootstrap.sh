
### Dependencies ###
CONTAINER_NAME="dependencies"
DEPENDENCIES=$CONTAINER_REGISTRY_URL_FULL/$CONTAINER_NAME:$GIT_COMMIT_HASH

cd /workspace && docker build --target $CONTAINER_NAME \
  -f /workspace/Dockerfile \
  -t $DEPENDENCIES \
  .

docker push $DEPENDENCIES

### Spark Master ###
# Builds the Spark image with support for Kubernetes Operator
# Loads the custom Jars in the Project
# Loads the custom Spark Configs in the Project

SPARK_BASE=$CONTAINER_REGISTRY_URL_FULL/spark:$GIT_COMMIT_HASH
SPARK_PYTHON=$CONTAINER_REGISTRY_URL_FULL/spark-py:$GIT_COMMIT_HASH
bash $SPARK_HOME/bin/docker-image-tool.sh -r $CONTAINER_REGISTRY_URL_FULL -t $GIT_COMMIT_HASH $SPARK_HOME/kubernetes/dockerfiles/spark/Dockerfile build
bash $SPARK_HOME/bin/docker-image-tool.sh -r $CONTAINER_REGISTRY_URL_FULL -t $GIT_COMMIT_HASH -p $SPARK_HOME/kubernetes/dockerfiles/spark/bindings/python/Dockerfile build

#TODO set the user uid?

docker push $SPARK_BASE
docker push $SPARK_PYTHON




ALL_SPARK_IMAGE=$CONTAINER_REGISTRY_URL_FULL/all-spark:$GIT_COMMIT_HASH

docker build \
  -f /workspace/Dockerfile \
  -t $ALL_SPARK_IMAGE \
  .

docker push $ALL_SPARK_IMAGE
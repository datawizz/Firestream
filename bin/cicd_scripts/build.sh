

set -e

###############################################################################
### Build Artifacts                                                         ###
###############################################################################

# for JVM services, build the fat jar

# /workspace/scala/transform/target/scala-2.13/kafka-streams-stateful-processing-assembly-1.0.jar



###############################################################################
### Build Docker Images                                                     ###
###############################################################################

## Build each of the Dockerfiles in the project


# TODO remove the "k3d" portion of the image name for more compatibility. This will become relevant when we start using a cloud registry

### Dependencies ###
BASE_IMAGE=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/dependencies:$GIT_COMMIT_HASH

cd /workspace && docker build --target dependencies \
  -f /workspace/Dockerfile \
  -t k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/dependencies:$GIT_COMMIT_HASH \
  .

docker push $BASE_IMAGE

### Spark Master ###
# Builds the Spark image with support for Kubernetes Operator
# Loads the custom Jars in the Project
# Loads the custom Spark Configs in the Project

SPARK_BASE=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/spark:$GIT_COMMIT_HASH
SPARK_PYTHON=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/spark-py:$GIT_COMMIT_HASH
bash $SPARK_HOME/bin/docker-image-tool.sh -r k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT -t $GIT_COMMIT_HASH $SPARK_HOME/kubernetes/dockerfiles/spark/Dockerfile build
bash $SPARK_HOME/bin/docker-image-tool.sh -r k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT -t $GIT_COMMIT_HASH -p $SPARK_HOME/kubernetes/dockerfiles/spark/bindings/python/Dockerfile build

#TODO set the user uid?

docker push $SPARK_BASE
docker push $SPARK_PYTHON


### Spark Thrift Server ###
# Exposes a Thrift JDBC entrypoint vai a HIVE2 API port
# Enables execution of jobs on a Spark Cluster using spark://
# SPARK_THRIFT_SERVER=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/spark_thrift_server:$GIT_COMMIT_HASH 
# cd /workspace/charts/spark_thrift_server && docker build \
#   --build-arg BASE_IMAGE=$BASE_IMAGE \
#   -t $SPARK_THRIFT_SERVER \
#   .

# docker push $SPARK_THRIFT_SERVER



# ALL_SPARK=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/all_spark:$GIT_COMMIT_HASH



#/workspace/charts/spark_thrift_server/Dockerfile

# cd $SPARK_HOME && docker build \
#   --build-arg BASE_IMAGE=$BASE_IMAGE \
#   -t $SPARK_HISTORY_SERVER \
#   .


### Spark History Server ###
SPARK_HISTORY_SERVER=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/spark_history_server:$GIT_COMMIT_HASH

cd /workspace/charts/spark_history_server && docker build \
  --build-arg BASE_IMAGE=$BASE_IMAGE \
  --build-arg SPARK_HOME=$SPARK_HOME \
  -t $SPARK_HISTORY_SERVER \
  .
docker push $SPARK_HISTORY_SERVER




# ### Websocket Middleware ###
# WEBSOCKET_MIDDLEWARE=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/websocket_middleware:$GIT_COMMIT_HASH

# cd /workspace/src/services/api/websocket_middleware && docker build \
#   --build-arg BASE_IMAGE=$BASE_IMAGE \
#   -t $WEBSOCKET_MIDDLEWARE \
#   .

# docker push $WEBSOCKET_MIDDLEWARE


# ### Hive Metastore ###
# HIVE_METASTORE=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/hive_metastore:$GIT_COMMIT_HASH

# cd /workspace/charts/hive_metastore/docker && docker build \
#   --build-arg BASE_IMAGE=$BASE_IMAGE \
#   -t  $HIVE_METASTORE \
#   .

# docker push $HIVE_METASTORE

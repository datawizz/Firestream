

set -e

###############################################################################
### Docker Registry                                                         ###
###############################################################################

# Find and test the configured Docker Registry
CONTAINER_REGISTRY_PREFIX="k3d-fireworks."
CONTAINER_REGISTRY_URL="localhost"
CONTAINER_REGISTRY_URL_FULL=$CONTAINER_REGISTRY_PREFIX$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT


###############################################################################
### Build Artifacts                                                         ###
###############################################################################

# for JVM services, build the fat jar

# /workspace/scala/transform/target/scala-2.13/kafka-streams-stateful-processing-assembly-1.0.jar
#cd /workspace/submodules/the-fireworks-company/spark && build/mvn -Pyarn -Phadoop-2.7 -Phive -Phive-thriftserver -DskipTests clean package

###############################################################################
### Build Docker Images                                                     ###
###############################################################################

## Build each of the Dockerfiles in the project


# TODO remove the "k3d" portion of the image name for more compatibility. This will become relevant when we start using a cloud registry

### Dependencies ###
CONTAINER_NAME="dependencies"
DEPENDENCIES=$CONTAINER_REGISTRY_URL_FULL/$CONTAINER_NAME:$GIT_COMMIT_HASH

cd /workspace && docker build --target $CONTAINER_NAME \
  -f /workspace/Dockerfile \
  -t $DEPENDENCIES \
  .

docker push $DEPENDENCIES




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

# #!/bin/bash

# set -e

# ###############################################################################
# ### Docker Registry                                                         ###
# ###############################################################################

# # Find and test the configured Docker Registry

# ###############################################################################
# ### Build Docker Images                                                     ###
# ###############################################################################

# ## Build each of the Dockerfiles in the project


# # Environment variables
# # CONTAINER_REGISTRY_URL="your_registry_url"
# # CONTAINER_REGISTRY_PORT="your_registry_port"
# # GIT_COMMIT_HASH="your_git_commit_hash"

# # Function to build and push Docker images
# build_and_push_docker_image() {
#   local container_name=$1
#   local dockerfile=$2
#   local git_commit_hash=$3
#   local _IMAGE_TAG="${CONTAINER_REGISTRY_URL}:${CONTAINER_REGISTRY_PORT}/${container_name}:${git_commit_hash}"
#   local _REPO="${CONTAINER_REGISTRY_URL}:${CONTAINER_REGISTRY_PORT}/${container_name}"

#   echo "Building $_IMAGE_TAG"
#   cd /workspace && docker build --target "$container_name" --progress=plain \
#     -f /workspace/docker/${dockerfile} \
#     --build-arg REPO="$_REPO" \
#     --build-arg IMAGE_TAG="${git_commit_hash}" \
#     -t "$_IMAGE_TAG" \
#     .
#   docker push "$_IMAGE_TAG"
# }



# # # Build and push Dependencies
# # echo "Building Docker Images for Dependencies"
# # build_and_push_docker_image "dependencies" "Dockerfile" "$GIT_COMMIT_HASH"

# # # Build and push Dependencies
# # echo "Building CICD Container"
# # build_and_push_docker_image "devcontainer" "Dockerfile" "$GIT_COMMIT_HASH"




# ### SPARK ###

# build_and_push_spark() {
#   # Define the image tag, usually just the git commit hash or a version identifier
#   IMAGE_TAG="${GIT_COMMIT_HASH}"

#   # Define the full repository name, including the registry URL and port
#   _REPO="${CONTAINER_REGISTRY_URL}:${CONTAINER_REGISTRY_PORT}"

#   cd "$SPARK_HOME"

#   # Build the Docker image using the specified repository and tag
#   ./bin/docker-image-tool.sh -r "$_REPO" -t "$IMAGE_TAG" build

#   # Push the Docker image to the specified repository with the tag
#   ./bin/docker-image-tool.sh -r "$_REPO" -t "$IMAGE_TAG" push
# }


# # Build and push Spark Master
# echo "Building Docker Images for All Spark"
# build_and_push_docker_image "all_spark" "Dockerfile" "$GIT_COMMIT_HASH"




# # # Builds the Spark image with support for Kubernetes Operator
# # # Loads the custom Jars in the Project
# # # Loads the custom Spark Configs in the Project

# # SPARK_BASE=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/spark:$GIT_COMMIT_HASH
# # SPARK_PYTHON=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/spark-py:$GIT_COMMIT_HASH
# # bash $SPARK_HOME/bin/docker-image-tool.sh -r k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT -t $GIT_COMMIT_HASH $SPARK_HOME/kubernetes/dockerfiles/spark/Dockerfile build
# # bash $SPARK_HOME/bin/docker-image-tool.sh -r k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT -t $GIT_COMMIT_HASH -p $SPARK_HOME/kubernetes/dockerfiles/spark/bindings/python/Dockerfile build

# # #TODO set the user uid?

# # docker push $SPARK_BASE
# # docker push $SPARK_PYTHON


# # ### Spark Thrift Server ###
# # # Exposes a Thrift JDBC entrypoint vai a HIVE2 API port
# # # Enables execution of jobs on a Spark Cluster using spark://
# # # SPARK_THRIFT_SERVER=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/spark_thrift_server:$GIT_COMMIT_HASH
# # # cd /workspace/charts/spark_thrift_server && docker build \
# # #   --build-arg BASE_IMAGE=$BASE_IMAGE \
# # #   -t $SPARK_THRIFT_SERVER \
# # #   .

# # # docker push $SPARK_THRIFT_SERVER



# # # ALL_SPARK=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/all_spark:$GIT_COMMIT_HASH



# # #/workspace/charts/spark_thrift_server/Dockerfile

# # # cd $SPARK_HOME && docker build \
# # #   --build-arg BASE_IMAGE=$BASE_IMAGE \
# # #   -t $SPARK_HISTORY_SERVER \
# # #   .


# # ### Spark History Server ###
# # SPARK_HISTORY_SERVER=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/spark_history_server:$GIT_COMMIT_HASH

# # cd /workspace/charts/spark_history_server && docker build \
# #   --build-arg BASE_IMAGE=$BASE_IMAGE \
# #   --build-arg SPARK_HOME=$SPARK_HOME \
# #   -t $SPARK_HISTORY_SERVER \
# #   .
# # docker push $SPARK_HISTORY_SERVER




# # # ### Websocket Middleware ###
# # # WEBSOCKET_MIDDLEWARE=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/websocket_middleware:$GIT_COMMIT_HASH

# # # cd /workspace/src/services/api/websocket_middleware && docker build \
# # #   --build-arg BASE_IMAGE=$BASE_IMAGE \
# # #   -t $WEBSOCKET_MIDDLEWARE \
# # #   .

# # # docker push $WEBSOCKET_MIDDLEWARE


# # # ### Hive Metastore ###
# # # HIVE_METASTORE=k3d-$COMPOSE_PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/hive_metastore:$GIT_COMMIT_HASH

# # # cd /workspace/charts/hive_metastore/docker && docker build \
# # #   --build-arg BASE_IMAGE=$BASE_IMAGE \
# # #   -t  $HIVE_METASTORE \
# # #   .

# # # docker push $HIVE_METASTORE



#!/bin/bash

set -e

# Configuration
REGISTRY="k3d-firestream.localhost:5055"
IMAGE_NAME="myapp"  # Change this to your application name
TAG="latest"

# Build the image
echo "Building image..."
docker build \
  -f /workspace/src/deployment/packages/spark/Dockerfile \
  -t ${REGISTRY}/${IMAGE_NAME}:${TAG} .

# Push to registry
echo "Pushing to registry..."
docker push ${REGISTRY}/${IMAGE_NAME}:${TAG}

echo "Done! Image is available at ${REGISTRY}/${IMAGE_NAME}:${TAG}"




# k3d-firestream.localhost:5055/myapp:latest

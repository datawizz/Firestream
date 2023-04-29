



###############################################################################
### Build Artifacts                                                         ###
###############################################################################

# for JVM services, build the fat jar

# /workspace/scala/transform/target/scala-2.13/kafka-streams-stateful-processing-assembly-1.0.jar



###############################################################################
### Build Docker Images                                                     ###
###############################################################################

## Build each of the Dockerfiles in the project

# Build the dependencies and make it available on the local Docker registry
cd /workspace && docker build --target dependencies \
  -f /workspace/Dockerfile \
  -t $CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/dependencies:$GIT_COMMIT_HASH \
  .
docker push $CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT/dependencies:$GIT_COMMIT_HASH

# TODO build the rest of the images in the project?
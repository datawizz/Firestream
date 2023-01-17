#!/bin/bash

### BOOTSTRAP ###

# This script bootstraps the project

# 1. Setup a Kind cluster
# 2. Setup networking proxy into Kind DNS (exposes internal services to the devcotainer, used in testing and development)
# 3. Create a local Docker Registry to store images built in the project
# 4. Setup Bitnami as a source in Helm, then install Kafka
# 5. Use Scala Build Tools to build a custom Spark function as a fat Jar
# 6. Build a Spark Image with the Jar included
# 5. Print to console information to connect

###############################################################################
### 1. Ensure Docker is available                                           ###
###############################################################################

# Assert that each command finishes without error
# set -e

# Start Docker
echo "Starting Docker"
# bash /usr/local/share/docker-init.sh
#Ensure it is started
docker --version

###############################################################################
### 2. KinD (Kubernetes in Docker) Cluster                                  ###
###############################################################################

# The project's KinD cluster is used for DNS resolution and needs to be available

# Define the cluster's name
cluster_name="fireworks"

# Check if the project's kind cluster is running
existing_cluster=$(kind get clusters | grep "$cluster_name")

if [ -z "$existing_cluster" ]; then
  echo "the cluster is not running, create it"
  # # Create a KinD cluster on the host's Docker Engine
  kind create cluster --config=/workspace/k8/kind/values.yaml --name $cluster_name
else
  echo "the cluster is running, attach to it"
  # TODO don't delete the cluster and recreate it, attach to it by exporting kubeconfig and importing to kubectl
  kind delete cluster --name $cluster_name
  kind create cluster --config=/workspace/k8/kind/values.yaml --name $cluster_name
fi

# Wait a moment for the cluster to come up
echo "Sleeping for 20 seconds to let KinD initialize"
sleep 20

###############################################################################
### 3. Resolve DNS through Kubernetes Control Plane                         ###
###############################################################################

# Find the IP of the Kind Control Plane Node (dynamically assigned with each cluster restart)
export KIND_IP=$(docker container inspect fireworks-control-plane --format '{{ .NetworkSettings.Networks.kind.IPAddress }}')
echo $KIND_IP


# TODO this needs to be idempotent as the second run after a devcontainer rebuild has conflicts with the existing config

# Set the route for Services inside the cluster
# This is also used for the CoreDNS service
sudo ip route add 10.96.0.0/16 via $KIND_IP

# Set the route for services in
sudo ip route add 10.244.0.0/16 via $KIND_IP

# # Route ALL THE THINGS
# sudo ip route add 10.0.0.0/8 via $KIND_IP

# sudo ip route add 10.100.0.0/24 via $KIND_IP

# TODO try to ping outside resources
# ping Google.com

###############################################################################
### 3. Local Docker Registry                                                ###
###############################################################################

# export DOCKER_PORT=5000
# export ARTIFACT_REPO=localhost:5000

# TODO make this idempotent and clear the registry from previous runs?

# Setup Docker Registry
printf "\nInstalling Local Docker Registry...\n"
docker pull registry

printf "\nStarting Local Docker Registry...\n\n"

reg_name='fireworks-registry'
reg_port='5000'
running="$(docker inspect -f '{{.State.Running}}' "${reg_name}" 2>/dev/null || true)"
if [ "${running}" != 'true' ]; then
  docker run \
    -d --restart=always -p "${reg_port}:5000" --name "${reg_name}" \
    registry:2
fi

# Connect registry running in Docker to the Kind network used in the cluster
docker network connect "kind" "${reg_name}"


###############################################################################
### Build Docker Images                                                     ###
###############################################################################

# export DOCKER_HOST=tcp://127.0.0.1:2375

# Build the dependencies and make it available on the local Docker registry
docker build --target dependencies -f /workspace/Dockerfile -t localhost:5000/dependencies:latest .
docker push localhost:5000/dependencies:latest

# Build the Spark SQL Structured Streaming Stateful container
docker build -f /workspace/services/scala/spark_structured_streaming_stateful/Dockerfile -t localhost:5000/spark_structured_streaming_stateful:latest .



###############################################################################
### Build Artifacts                                                         ###
###############################################################################

# # # cd 

# # # /workspace/scala/transform/target/scala-2.13/kafka-streams-stateful-processing-assembly-1.0.jar







# # ###############################################################################
# # ### K8 Services                                                             ###
# # ###############################################################################

# Install Bitnami repo
helm repo add bitnami https://charts.bitnami.com/bitnami

# Install Kafka using local Values.yaml overrides
helm install kafka bitnami/kafka -f /workspace/k8/kafka/values.yaml

# Install Spark using local Values.yaml overrides
helm install spark bitnami/spark -f /workspace/k8/spark/values.yaml




# # ###############################################################################
# # ### Sleep                                                                   ###
# # ###############################################################################

# # Keep the container alive indefinitely
# Only needed if this is the entrypoint script of the project
# sleep infinity
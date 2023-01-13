
# This script bootstraps the project


# 1. Setup a Kind cluster
# 2. Setup networking proxy into Kind DNS (exposes internal services to the devcotainer, used in testing and development)
# 3. Create a local Docker Registry to store images built in the project
# 4. Setup Bitnami as a source in Helm, then install Kafka
# 5. Use Scala Build Tools to build a custom Spark function as a fat Jar
# 6. Build a Spark Image with the Jar included
# 5. Print to console information to connect

#!/bin/bash

# Attempt to ping google.com
ping -c 1 -W 2 google.com &> /dev/null
if [ $? -ne 0 ]; then
  # Check if Kind is accessible
  if ! kind get clusters; then
    # If not, run delete kind
    kind delete cluster
  fi
fi


###############################################################################
### 1. Kind K8 Cluster                                                      ###
###############################################################################

# Create a KinD cluster on the host's Docker Engine
kind create cluster --config=/workspace/charts/1_kind/values.yaml

###############################################################################
### 2. Resolve DNS through Kubernetes Control Plane                         ###
###############################################################################


# Find the IP of the Kind Control Plane Node (dynamically assigned with each cluster restart)
export KIND_IP=$(docker container inspect kind-control-plane --format '{{ .NetworkSettings.Networks.kind.IPAddress }}')
echo $KIND_IP
# Set the route for Services inside the cluster
# This is also used for the CoreDNS service
sudo ip route add 10.96.0.0/16 via $KIND_IP

# Set the route for services in
sudo ip route add 10.244.0.0/16 via $KIND_IP

# # Route ALL THE THINGS
# sudo ip route add 10.0.0.0/8 via $KIND_IP

# sudo ip route add 10.100.0.0/24 via $KIND_IP


###############################################################################
### 3. Local Docker Registry                                                ###
###############################################################################

# export DOCKER_PORT=5000
# export ARTIFACT_REPO=localhost:5000

# Setup Docker Registry
printf "\nInstalling Local Docker Registry...\n"
docker pull registry

printf "\nStarting Local Docker Registry...\n\n"


reg_name='kind-registry'
reg_port='5000'
running="$(docker inspect -f '{{.State.Running}}' "${reg_name}" 2>/dev/null || true)"
if [ "${running}" != 'true' ]; then
  docker run \
    -d --restart=always -p "${reg_port}:5000" --name "${reg_name}" \
    registry:2
fi

# Connect registry running in Docker to the Kind network used in the cluster
docker network connect "kind" "kind-registry"


# Build each of the project's Dockerfiles and push the images to the local registry

# Build the Spark Driver/Executor


# # Build the dashboard
# docker build -f /workspaces/k8-spark-kafka/charts/6_dashboard/Dockerfile ./charts/6_dashboard


# ###############################################################################
# ### Build Images                                                            ###
# ###############################################################################

# cd 

# /workspace/scala/transform/target/scala-2.13/kafka-streams-stateful-processing-assembly-1.0.jar




# ###############################################################################
# ### Build Images                                                            ###
# ###############################################################################

# Build the images which will be used in the Spark Cluster




# ###############################################################################
# ### K8 Services                                                             ###
# ###############################################################################

# Install Bitnami repo
helm repo add bitnami https://charts.bitnami.com/bitnami

# Install Kafka using local Values.yaml overrides
helm install kafka bitnami/kafka -f /workspace/charts/2_kafka/values.yaml

# Install Spark using local Values.yaml overrides
helm install spark bitnami/spark -f /workspace/charts/3_spark/values.yaml

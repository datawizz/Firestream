


# Bootstraps a Kubernetes Cluster in Docker


###############################################################################
### 2. KinD (Kubernetes in Docker) Cluster                                  ###
###############################################################################

#TODO make this more flexible

# Check if the project's kind cluster is running
existing_cluster=$(kind get clusters | grep "$COMPOSE_PROJECT_NAME") || existing_cluster=""

if [ -z "$existing_cluster" ]; then
  echo "the cluster is not running, create it"
  # # Create a KinD cluster on the host's Docker Engine
  kind create cluster --config=/workspace/docker/kind/values.yaml --name $COMPOSE_PROJECT_NAME
else
  echo "the cluster is running, delete it and recreate it"
  # TODO don't delete the cluster and recreate it, attach to it by exporting kubeconfig and importing to kubectl
  kind delete cluster --name $COMPOSE_PROJECT_NAME
  kind create cluster --config=/workspace/docker/kind/values.yaml --name $COMPOSE_PROJECT_NAME
fi

# Wait a moment for the cluster to come up
# echo "Sleeping for 20 seconds to let KinD initialize"
# sleep 20


echo "Waiting for Kind cluster to be ready..."
kubectl wait --for=condition=ready pod --all -n kube-system --timeout=2m
echo "Kind cluster is now ready"




### Resolve DNS through Kubernetes Control Plane

# Find the IP of the Kind Control Plane Node (dynamically assigned with each cluster restart)
export KIND_IP=$(docker container inspect fireworks-control-plane --format '{{ .NetworkSettings.Networks.kind.IPAddress }}')
echo $KIND_IP


# TODO this needs to be idempotent as the second run after a devcontainer rebuild has conflicts with the existing config

# Set the route for Services inside the cluster
# This is also used for the CoreDNS service

sudo ip route add 10.96.0.0/16 via $KIND_IP || (echo "Route already exists")


# Set the route for services in
sudo ip route add 10.244.0.0/16 via $KIND_IP  || (echo "Route already exists")

# # Route ALL THE THINGS
# sudo ip route add 10.0.0.0/8 via $KIND_IP

# sudo ip route add 10.100.0.0/24 via $KIND_IP

# TODO try to ping outside resources
# ping Google.com

###############################################################################
### 3. Local Docker Registry                                                ###
###############################################################################

#TODO this needs to be idempotent as the second run after a devcontainer rebuild has conflicts with the existing config
# If the script is being run in a container, and it has access to Docker engine, 
# build the images and push to the configured registry.



# Setup Docker Registry
if [ "$DEPLOYMENT_MODE" = "development" ]; then
  printf "\nInstalling Local Docker Registry...\n"
  docker pull registry

  printf "\nStarting Local Docker Registry...\n\n"

  # Idempotent code to create a registry container
  REGISTRY_NAME="${COMPOSE_PROJECT_NAME}-registry"

  # Check if the container exists and stop and remove it if it does
  if [ "$(docker ps -a -q -f name="${REGISTRY_NAME}" --format '{{.Names}}')" = "${REGISTRY_NAME}" ]; then
    docker stop "${REGISTRY_NAME}"
    docker rm "${REGISTRY_NAME}"
  fi

  # Create a new registry container
  docker run \
  -d --restart=always -p "${CONTAINER_REGISTRY_PORT}:${CONTAINER_REGISTRY_PORT}" --name "${REGISTRY_NAME}" \
  registry:2


  # connect the registry to the cluster network if not already connected
  if [ "$(docker inspect -f='{{json .NetworkSettings.Networks.kind}}' "${REGISTRY_NAME}")" = 'null' ]; then
    docker network connect "kind" "${REGISTRY_NAME}"
  fi
fi


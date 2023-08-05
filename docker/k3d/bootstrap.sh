#!/bin/bash
set -e
###############################################################################
### 2. K3D - K3S in Docker                                                  ###
###############################################################################

# Bootstraps a Kubernetes Cluster in Docker (K3D) on the host's Docker Engine

CLUSTER_NAME=$COMPOSE_PROJECT_NAME

# Check if the registry exists
if ! k3d registry list | grep -q $CLUSTER_NAME.$CONTAINER_REGISTRY_URL; then
  # Create the k3d registry
  echo "Creating k3d registry named $CLUSTER_NAME.$CONTAINER_REGISTRY_URL..."
  k3d registry create $CLUSTER_NAME.$CONTAINER_REGISTRY_URL --port $CONTAINER_REGISTRY_PORT
else
  echo "k3d registry named $CLUSTER_NAME.$CONTAINER_REGISTRY_URL already exists."
fi

# Check if the cluster exists
if k3d cluster list | grep -q $CLUSTER_NAME; then
  # If the cluster exists, delete it before creating a new one
  echo "k3d cluster named $CLUSTER_NAME already exists. Deleting..."
  k3d cluster delete $CLUSTER_NAME
fi

# Create the k3d cluster
echo "Creating k3d cluster named $CLUSTER_NAME..."
k3d cluster create $CLUSTER_NAME --api-port 6550 -p "80:80@loadbalancer" -p "443:443@loadbalancer" --registry-use k3d-$CLUSTER_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT


# Configure kubectl to use the MYCLUSTER
echo "Configuring kubectl to use k3d cluster named $CLUSTER_NAME..."
kubectl config use-context "k3d-$CLUSTER_NAME"

# Wait for COREDNS to be available
kubectl wait --namespace=kube-system --for=condition=available --timeout=300s --all deployments

echo $(kubectl cluster-info)


# Configure Self Signed Certificates
bash /workspace/bin/cicd_scripts/configure-tls-debian.sh

# Load TLS Certificates into the cluster as a secret
kubectl create secret tls $TLS_SECRET --cert=$HOME/certs/server.crt --key=$HOME/certs/server.key

#TODO: This is a hack to get the ingress to work. It should be removed once the ingress is configured properly
sleep 10

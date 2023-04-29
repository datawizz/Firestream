#!/bin/bash
set -e
###############################################################################
### 2. K3D - K3S in Docker                                                  ###
###############################################################################

# Bootstraps a Kubernetes Cluster in Docker (K3D) on the host's Docker Engine

CLUSTER_NAME=$COMPOSE_PROJECT_NAME

# Check if k3d cluster named $CLUSTER_NAME already exists
if ! k3d cluster list | grep -q $CLUSTER_NAME; then
  # Create the k3d cluster
  echo "Creating k3d cluster named $CLUSTER_NAME..."
  k3d cluster create $CLUSTER_NAME --network fireworks #--config /workspace/docker/k3d/values.yaml
else
  echo "k3d cluster named $CLUSTER_NAME already exists."
  k3d cluster delete $CLUSTER_NAME
  k3d cluster create $CLUSTER_NAME --network fireworks
fi

# Configure kubectl to use the MYCLUSTER
echo "Configuring kubectl to use k3d cluster named $CLUSTER_NAME..."
kubectl config use-context "k3d-$CLUSTER_NAME"

# Wait for COREDNS to be available
kubectl wait --namespace=kube-system --for=condition=available --timeout=300s deployment/coredns

echo kubectl cluster-info

### Resolve DNS through Kubernetes Control Plane ###

# Find the IP of the k3d server node (dynamically assigned with each cluster restart)
export K3D_SERVER_IP=$(docker container inspect k3d-$COMPOSE_PROJECT_NAME-server-0 --format '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}')
echo $K3D_SERVER_IP

# Check if the route already exists and set the route for Services inside the cluster
# This is also used for the CoreDNS service
sudo ip route | grep -q "10.43.0.0/16" || sudo ip route add 10.43.0.0/16 via $K3D_SERVER_IP

# Check if the route already exists and set the route for Pod networks
sudo ip route | grep -q "10.42.0.0/16" || sudo ip route add 10.42.0.0/16 via $K3D_SERVER_IP

### Update resolv.conf with dynamic Kubernetes DNS IP

# Find the IP of the CoreDNS service in your Kubernetes cluster
KUBERNETES_DNS_IP=$(kubectl get svc -n kube-system kube-dns -o jsonpath='{.spec.clusterIP}')
echo $KUBERNETES_DNS_IP

# Update the resolv.conf file inside your development container with the new DNS settings
sudo echo -e "search svc.cluster.local cluster.local\nnameserver $KUBERNETES_DNS_IP\noptions edns0 trust-ad" | sudo tee /etc/resolv.conf




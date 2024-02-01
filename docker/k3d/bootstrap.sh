#!/bin/bash

###############################################################################
### Kubernetes in Docker (K3D) Bootstrap                                    ###
###############################################################################
set -e


ensure_kube_directory() {
  mkdir -p $HOME/.kube
}

check_registry() {
  if k3d registry list | grep -q $PROJECT_NAME.$CONTAINER_REGISTRY_URL; then
    echo "k3d registry named $PROJECT_NAME.$CONTAINER_REGISTRY_URL already exists."
  else
    echo "Creating k3d registry named $PROJECT_NAME.$CONTAINER_REGISTRY_URL..."
    k3d registry create $PROJECT_NAME.$CONTAINER_REGISTRY_URL --port $CONTAINER_REGISTRY_PORT
  fi
}

check_cluster() {
  if k3d cluster list | grep -q $PROJECT_NAME; then
    reconnect_cluster
  else
    create_cluster
  fi
}

reconnect_cluster() {
  echo "k3d cluster named $PROJECT_NAME already exists. Connecting..."
  k3d kubeconfig write $PROJECT_NAME --kubeconfig-switch-context
  mv $HOME/.config/k3d/kubeconfig-$PROJECT_NAME.yaml $HOME/.kube/config
  kubectl config --kubeconfig=$HOME/.kube/config use-context k3d-$PROJECT_NAME
  test_kubectl
}

create_cluster() {
  echo "Creating k3d cluster named $PROJECT_NAME..."
  k3d cluster create $PROJECT_NAME --api-port 6550 -p "80:80@loadbalancer" -p "443:443@loadbalancer" --registry-use k3d-$PROJECT_NAME.$CONTAINER_REGISTRY_URL:$CONTAINER_REGISTRY_PORT
  echo "Configuring kubectl to use k3d cluster named $PROJECT_NAME..."
  kubectl config use-context "k3d-$PROJECT_NAME"
  wait_for_coredns
}

delete_cluster() {
  echo "Deleting k3d cluster named $PROJECT_NAME..."
  k3d cluster delete $PROJECT_NAME
}

test_kubectl() {
  kubectl_version_output=$(kubectl version --output=json)
  client_git_version=$(echo "$kubectl_version_output" | jq -r '.clientVersion.GitVersion')
  server_git_version=$(echo "$kubectl_version_output" | jq -r '.serverVersion.GitVersion')

  if [ -n "$client_git_version" ] && [ -n "$server_git_version" ]; then
    echo "Both client and server are accessible, and GitVersion is populated."
  else
    echo "Error: GitVersion is not populated for client and/or server."
    exit 1
  fi
}

wait_for_coredns() {
  kubectl wait --namespace=kube-system --for=condition=available --timeout=300s --all deployments
}





configure_certificates() {
  # Configure Self Signed Certificates
  bash /workspace/bin/cicd_scripts/configure-tls-debian.sh
}

load_tls_certificates() {
  # Check if the secret already exists
  if kubectl get secret $TLS_SECRET > /dev/null 2>&1; then
    echo "TLS secret $TLS_SECRET already exists."
  else
    # If the secret does not exist, configure self-signed certificates
    configure_certificates
    # Load TLS Certificates into the cluster as a secret
    kubectl create secret tls $TLS_SECRET --cert=$HOME/certs/server.crt --key=$HOME/certs/server.key
    echo "TLS secret $TLS_SECRET created."
  fi
}


hack_for_ingress() {
  #TODO: This is a hack to get the ingress to work. It should be removed once the ingress is configured properly
  sleep 10
}





patch_etc_hosts() {
  HOSTNAME=$(hostname)
  grep -q "$HOSTNAME" /etc/hosts
  if [ $? -eq 1 ]; then
    echo "127.0.0.1 $HOSTNAME" | sudo tee -a /etc/hosts > /dev/null
    echo "Hostname $HOSTNAME added to /etc/hosts"
  else
    echo "Hostname $HOSTNAME already exists in /etc/hosts"
  fi
}

set_ip_routes() {
  export K3D_SERVER_IP=$(docker container inspect k3d-$PROJECT_NAME-server-0 --format '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}')
  echo $K3D_SERVER_IP
  sudo ip route | grep -q "10.43.0.0/16" || sudo ip route add 10.43.0.0/16 via $K3D_SERVER_IP
  sudo ip route | grep -q "10.42.0.0/16" || sudo ip route add 10.42.0.0/16 via $K3D_SERVER_IP
}

update_resolv_conf() {
  KUBERNETES_DNS_IP=$(kubectl get svc -n kube-system kube-dns -o jsonpath='{.spec.clusterIP}')
  echo $KUBERNETES_DNS_IP
  sudo echo -e "search svc.cluster.local cluster.local\nnameserver $KUBERNETES_DNS_IP\noptions edns0 trust-ad" | sudo tee /etc/resolv.conf
}

test_dns_resolution() {
  wait_time=1
  max_attempts=4
  attempt=0

  while [ $attempt -lt $max_attempts ]
  do
    curl -s -I debian.org > /dev/null 2>&1
    if [ $? -eq 0 ]
    then
      echo "Connection was successful!"
      exit 0
    else
      echo "Connection failed, trying again in $wait_time seconds..."
      sleep $wait_time
      wait_time=$((wait_time*2))
      attempt=$((attempt+1))
    fi
  done

  echo "All attempts to connect have failed. Please check your network settings."
  exit 1
}



# Run
ensure_kube_directory
check_registry
check_cluster

# If "Clean" mode then delete the cluster and recreate it
if [ "$DEPLOYMENT_MODE" = "clean" ]; then
  delete_cluster
  create_cluster
fi

load_tls_certificates
hack_for_ingress

patch_etc_hosts
set_ip_routes
update_resolv_conf
test_dns_resolution


### Resolve DNS through Kubernetes Control Plane ###

# Patch the /etc/hosts with the host-uuid for codespacec compatibility
HOSTNAME=$(hostname)

# Check if the hostname already exists in /etc/hosts
grep -q "$HOSTNAME" /etc/hosts

if [ $? -eq 1 ]; then
  # If hostname not found, append it to /etc/hosts
  echo "127.0.0.1 $HOSTNAME" | sudo tee -a /etc/hosts > /dev/null
  echo "Hostname $HOSTNAME added to /etc/hosts"
else
  echo "Hostname $HOSTNAME already exists in /etc/hosts"
fi

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


# Test DNS resolution
bash /workspace/bin/commands/network_connection_test.sh
#!/bin/bash

# Exposes ports for all services in the project with external ports prefixed with 10000
# Used only in development mode!

mkdir -p /workspace/logs

if [ "$DEPLOYMENT_MODE" != "development" ]; then
  echo "Invalid deployment_mode $DEPLOYMENT_MODE."
  echo "only 'development' mode is supported for exposing ports using kubectl!"
  exit 1
fi

# Function to forward ports
forward_service_port() {
    local namespace=$1
    local service=$2
    local internal_port=$3
    local external_port=$((10000 + internal_port))

    echo "Forwarding service $service port $internal_port to $external_port"
    nohup kubectl port-forward --namespace $namespace svc/$service $external_port:$internal_port >> /workspace/logs/port_forwards.log 2>&1 &
}

# Get all services in the default namespace
services=$(kubectl get svc -n default -o jsonpath='{range .items[*]}{.metadata.name},{.spec.ports[*].port}{"\n"}{end}')

echo "Starting port forwarding for all services..."

# Process each service and its ports
while IFS=',' read -r service ports; do
    if [ ! -z "$service" ] && [ ! -z "$ports" ]; then
        # Split ports string into array
        IFS=' ' read -ra port_array <<< "$ports"

        # Forward each port
        for port in "${port_array[@]}"; do
            if [ ! -z "$port" ]; then
                forward_service_port "default" "$service" "$port"
            fi
        done
    fi
done <<< "$services"

echo "Port forwarding setup complete. Check /workspace/logs/port_forwards.log for details"

# List all forwarded ports
echo -e "\nForwarded Ports:"
ps aux | grep "kubectl port-forward" | grep -v grep

# Optional: Save list of forwarded ports to a file
ps aux | grep "kubectl port-forward" | grep -v grep > /workspace/logs/active_forwards.log

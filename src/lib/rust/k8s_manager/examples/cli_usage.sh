#!/bin/bash
# Example: Using the k8s-manager CLI
#
# This script demonstrates common CLI usage patterns

set -e

echo "=== K8s Manager CLI Examples ==="
echo

# Check if k8s-manager is available
if ! command -v k8s-manager &> /dev/null; then
    echo "k8s-manager not found. Running with cargo..."
    K8S_CMD="cargo run --"
else
    echo "Using installed k8s-manager"
    K8S_CMD="k8s-manager"
fi

# Function to run command with echo
run_cmd() {
    echo "$ $*"
    eval "$*"
    echo
}

echo "1. List existing clusters"
run_cmd "$K8S_CMD cluster list"

echo "2. Create a basic cluster"
run_cmd "$K8S_CMD cluster create example-basic --basic"

echo "3. Get cluster information"
run_cmd "$K8S_CMD cluster info example-basic"

echo "4. Create a development cluster with all features"
run_cmd "$K8S_CMD cluster create example-dev --dev-mode --port-offset 30000"

echo "5. View diagnostics"
run_cmd "$K8S_CMD diagnostics --nodes --pods"

echo "6. Stop the development cluster"
run_cmd "$K8S_CMD cluster stop example-dev"

echo "7. Start it again"
run_cmd "$K8S_CMD cluster start example-dev"

echo "8. Delete the clusters"
run_cmd "$K8S_CMD cluster delete example-basic"
run_cmd "$K8S_CMD cluster delete example-dev"

echo "=== Examples completed ==="
echo
echo "Additional examples to try:"
echo "- Setup a fully configured cluster: $K8S_CMD cluster setup"
echo "- View logs: $K8S_CMD logs <pod-name> --follow"
echo "- Port forward a service: $K8S_CMD port-forward <service> --local-port 8080 --remote-port 80"
echo "- Get all diagnostics: $K8S_CMD diagnostics --all"

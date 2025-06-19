#!/bin/bash
# Example usage of k8s-manager image commands

echo "=== K8s Manager Image Commands Examples ==="
echo

# Ensure we have a k3d cluster and registry
echo "1. First, ensure you have a k3d cluster with registry:"
echo "   k8s-manager cluster create --name firestream"
echo "   k8s-manager registry create --name registry.localhost --port 5000"
echo

# Build an image
echo "2. Build a Docker image:"
echo "   k8s-manager image build --context . --tag myapp:latest"
echo

# Build with custom Dockerfile and build args
echo "3. Build with custom Dockerfile and build arguments:"
echo "   k8s-manager image build \\
     --context ./my-app \\
     --dockerfile Dockerfile.prod \\
     --tag myapp:v1.0.0 \\
     --build-arg VERSION=1.0.0 \\
     --build-arg BUILD_DATE=\$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
echo

# Push to k3d registry
echo "4. Push an existing image to k3d registry:"
echo "   k8s-manager image push myapp:latest"
echo

# Push with explicit registry
echo "5. Push to a specific registry:"
echo "   k8s-manager image push myapp:latest --registry registry.localhost:5000"
echo

# Build and push in one command
echo "6. Build and push in one command:"
echo "   k8s-manager image build-and-push \\
     --context . \\
     --tag myapp:latest \\
     --registry registry.localhost:5000"
echo

# Use in Kubernetes deployment
echo "7. After pushing, use the image in a Kubernetes deployment:"
echo "   kubectl create deployment myapp --image=registry.localhost:5000/myapp:latest"
echo

echo "=== Additional Notes ==="
echo "- The registry URL is auto-detected from k3d if not specified"
echo "- Images are automatically retagged for the k3d registry"
echo "- Build arguments can be passed with --build-arg KEY=VALUE"
echo "- Use --no-cache to force a fresh build"
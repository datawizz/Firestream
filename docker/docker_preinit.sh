#!/bin/bash

# Determine OS platform
OS_PLATFORM="$(uname -s)"

case "$OS_PLATFORM" in
    Linux*)     os=Linux;;
    Darwin*)    os=Mac;;
    *)          os="UNKNOWN:${OS_PLATFORM}"
esac

# Assert that Docker is available on the host
if ! command -v docker &> /dev/null; then
    echo "Docker must be installed to run this script."
    exit 1
fi

# Function to handle Linux-specific operations
handle_linux() {
    # Export the Docker Group ID for Linux
    export HOST_DOCKER_GID=$(getent group docker | cut -d: -f3)
    if [ -z "$HOST_DOCKER_GID" ]; then
        echo "Failed to get Docker Group ID."
        exit 1
    fi

    # Get the Machine ID for Linux
    export HOST_MACHINE_ID=$(cat /etc/machine-id)
}

# Function to handle Mac-specific operations
handle_mac() {
    # macOS Docker desktop manages group permissions differently.
    # Placeholder for macOS specific logic if needed.
    echo "Handling macOS specifics. Docker GID management not required."
    
    # Get the Machine ID for macOS
    export HOST_MACHINE_ID=$(sysctl -n kern.uuid)
}

# Invoke OS-specific functions
if [ "$os" = "Linux" ]; then
    handle_linux
elif [ "$os" = "Mac" ]; then
    handle_mac
else
    echo "Unsupported OS: $OS_PLATFORM"
    exit 1
fi

if [ -z "$HOST_MACHINE_ID" ]; then
    echo "Failed to get Machine ID."
    exit 1
fi

# Get the IP address of the host (used for end-to-end integration testing)
# This command works on both Linux and macOS
export HOST_IP=$(hostname -I 2>/dev/null || ipconfig getifaddr en0)
if [ -z "$HOST_IP" ]; then
    echo "Failed to get Host IP."
    exit 1
fi

# Get the logged in user ID and Group ID (compatible with both Linux and macOS)
export HOST_USER_ID=$(id -u)
export HOST_GROUP_ID=$(id -g)
if [ -z "$HOST_USER_ID" ] || [ -z "$HOST_GROUP_ID" ]; then
    echo "Failed to get User ID or Group ID."
    exit 1
fi

# Function to extract GPU vendor for Linux
extract_linux_gpu_vendor() {
    # Check for the presence of lspci command
    if command -v lspci &> /dev/null; then
        # Use lspci to find the GPU and extract the vendor name
        vendor=$(lspci | grep -E "VGA|3D" | sed -r 's/^.*: ([^:]*):.*$/\1/')
        echo "$vendor"
    else
        echo "lspci command not found, unable to extract GPU vendor."
        exit 1
    fi
}

# Function to extract GPU model for macOS
extract_macos_gpu_vendor() {
    # Use system_profiler to find the GPU and extract the model
    model=$(system_profiler SPDisplaysDataType | awk -F: '/Chipset Model/ {print $2}' | head -1 | sed 's/^ *//')
    echo "$model"
}

# Check OS and extract GPU information
if [ "$os" = "Linux" ]; then
    HOST_GPU_VENDOR=$(extract_linux_gpu_vendor)
    echo "GPU Vendor/Model: $HOST_GPU_VENDOR"
elif [ "$os" = "Mac" ]; then
    HOST_GPU_VENDOR=$(extract_macos_gpu_vendor)
    echo "GPU Vendor/Model: $HOST_GPU_VENDOR"
fi

# Check if GPU vendor/model was extracted
if [ -n "$HOST_GPU_VENDOR" ]; then
    export HOST_GPU_STATUS="true"
else
    echo "Failed to get GPU vendor/model."
    export HOST_GPU_STATUS="false"
fi

HOST_USERNAME=$(whoami)

# Create a temporary docker-compose file with environment variables and build arguments
TEMP_COMPOSE_FILE="docker/docker-compose.temp.yml"
{
  echo "version: '3.8'"
  echo "services:"
  echo "  devcontainer:"
  echo "    environment:"
  echo "      HOST_USER_ID: $HOST_USER_ID"
  echo "      HOST_GROUP_ID: $HOST_GROUP_ID"
  echo "      HOST_DOCKER_GID: $HOST_DOCKER_GID"
  echo "      HOST_MACHINE_ID: $HOST_MACHINE_ID"
  echo "      HOST_IP: $HOST_IP"
  echo "      HOST_GPU_STATUS: $HOST_GPU_STATUS"
  echo "      HOST_GPU_VENDOR: $HOST_GPU_VENDOR"
  echo "    build:"
  echo "      args:"
  echo "        HOST_USER_ID: $HOST_USER_ID"
  echo "        HOST_GROUP_ID: $HOST_GROUP_ID"
  echo "        HOST_GPU_STATUS: $HOST_GPU_STATUS"
  echo "        HOST_GPU_VENDOR: $HOST_GPU_VENDOR"
  echo "        HOST_USERNAME: $HOST_USERNAME"
} > $TEMP_COMPOSE_FILE


# Define the base and override file paths
BASE_FILE="docker/docker-compose.yml"
OVERRIDE_FILE="docker/docker-compose.gpu_override.yml"

# Combine Compose Files based on GPU_STATUS
if [ "$GPU_STATUS" = "true" ]; then
    docker compose -f $BASE_FILE -f $OVERRIDE_FILE -f $TEMP_COMPOSE_FILE config > docker/docker-compose.devcontainer.yml
else
    docker compose -f $BASE_FILE -f $TEMP_COMPOSE_FILE config > docker/docker-compose.devcontainer.yml
fi

# Remove the temporary docker-compose file
rm $TEMP_COMPOSE_FILE

echo "Pre-init script complete."
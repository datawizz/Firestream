#!/bin/bash
set -e

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

# Get the IP address of the host
export HOST_IP=$(hostname -I 2>/dev/null || ipconfig getifaddr en0)
if [ -z "$HOST_IP" ]; then
    echo "Failed to get Host IP."
    exit 1
fi

# Get the logged in user ID and Group ID
export HOST_USER_ID=$(id -u)
export HOST_GROUP_ID=$(id -g)
if [ -z "$HOST_USER_ID" ] || [ -z "$HOST_GROUP_ID" ]; then
    echo "Failed to get User ID or Group ID."
    exit 1
fi

# Extract GPU vendor for Linux
extract_linux_gpu_vendor() {
    if command -v lspci &> /dev/null; then
        # Simplify extraction logic to focus on identifying NVIDIA
        if lspci | grep -E "VGA|3D" | grep -iq "nvidia"; then
            echo "NVIDIA"
        else
            echo "Other"
        fi
    else
        echo "lspci command not found, unable to extract GPU vendor."
        exit 1
    fi
}

# Extract GPU model for macOS
extract_macos_gpu_vendor() {
    # Check for Metal API support, simplifying the assumption to Apple GPUs
    if system_profiler SPDisplaysDataType | grep -iq "Metal"; then
        echo "Apple Metal"
    else
        echo "Other"
    fi
}

# Extract GPU information
if [ "$os" = "Linux" ]; then
    HOST_GPU_VENDOR=$(extract_linux_gpu_vendor)
elif [ "$os" = "Mac" ]; then
    HOST_GPU_VENDOR=$(extract_macos_gpu_vendor)
fi
echo "GPU Vendor: $HOST_GPU_VENDOR"

# Set GPU_STATUS based on vendor detection
if [[ "$HOST_GPU_VENDOR" == "NVIDIA" || "$HOST_GPU_VENDOR" == "Apple Metal" ]]; then
    export HOST_GPU_STATUS="true"
else
    export HOST_GPU_STATUS="false"
fi

HOST_USERNAME=$(whoami)

# Function to set variable based on file existence
set_env_variable() {
  local example_file_path="../etc/.env.secrets.example"
  local expected_file_path="../etc/.env.secrets"

  # Check if the file exists
  if [ -e "$expected_file_path" ]; then
    # File exists, set the variable to the file path
    ENV_SECRETS_PATH="$expected_file_path"
  else
    # File does not exist, use the example secrets
    ENV_SECRETS_PATH="$example_file_path"
  fi

  # Export the variable for global use (optional)
  export ENV_SECRETS_PATH

  # For debugging or confirmation, you can display the variable value
  echo "ENV_SECRETS_PATH is set to: $ENV_SECRETS_PATH"
}

# To use this function, simply call it
set_env_variable

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
  echo "    env_file:"
  echo "      - $ENV_SECRETS_PATH"
} > $TEMP_COMPOSE_FILE

# Define the base and override file paths
BASE_FILE="docker/docker-compose.yml"

# Combine Compose Files based on GPU_STATUS
if [ "$HOST_GPU_VENDOR" = "NVIDIA" ]; then
    OVERRIDE_FILE="docker/docker-compose.gpu_nvidia.yml"
    docker compose -f $BASE_FILE -f $OVERRIDE_FILE -f $TEMP_COMPOSE_FILE config > docker/docker-compose.devcontainer.yml
else
    docker compose -f $BASE_FILE -f $TEMP_COMPOSE_FILE config > docker/docker-compose.devcontainer.yml
fi

# Remove the temporary docker-compose file
rm $TEMP_COMPOSE_FILE

echo "Pre-init script complete."
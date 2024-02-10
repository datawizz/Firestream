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

# Check for NVIDIA GPU (Linux) or any GPU on macOS (since macOS does not commonly use NVIDIA hardware for recent models)
export HOST_GPU_STATUS="false"
if [ "$os" = "Linux" ] && lspci | grep -i nvidia &> /dev/null; then
    HOST_GPU_STATUS="true"
elif [ "$os" = "Mac" ] && system_profiler SPDisplaysDataType | grep "Chipset Model:" &> /dev/null; then
    HOST_GPU_STATUS="true"
fi
if [ -z "$HOST_GPU_STATUS" ]; then
    echo "Failed to get GPU status."
    exit 1
fi

# Create a temporary docker-compose file
TEMP_COMPOSE_FILE="docker/docker-compose.temp.yml"
echo "services:" > $TEMP_COMPOSE_FILE
echo "  devcontainer:" >> $TEMP_COMPOSE_FILE
echo "    environment:" >> $TEMP_COMPOSE_FILE
echo "      HOST_USER_ID: $HOST_USER_ID" >> $TEMP_COMPOSE_FILE
echo "      HOST_GROUP_ID: $HOST_GROUP_ID" >> $TEMP_COMPOSE_FILE
echo "      HOST_DOCKER_GID: $HOST_DOCKER_GID" >> $TEMP_COMPOSE_FILE
echo "      HOST_MACHINE_ID: $HOST_MACHINE_ID" >> $TEMP_COMPOSE_FILE
echo "      HOST_IP: $HOST_IP" >> $TEMP_COMPOSE_FILE
echo "      HOST_GPU_STATUS: $HOST_GPU_STATUS" >> $TEMP_COMPOSE_FILE



# Combine Compose Files
BASE_FILE="docker/docker-compose.yml"
if [ "$GPU_STATUS" = "true" ]; then
    OVERRIDE_FILE="docker/docker-compose.gpu_override.yml"
    docker compose -f $BASE_FILE -f $OVERRIDE_FILE -f $TEMP_COMPOSE_FILE config > docker/docker-compose.devcontainer.yml
else
    docker compose -f $BASE_FILE -f $TEMP_COMPOSE_FILE config > docker/docker-compose.devcontainer.yml
fi

# Remove the temporary docker-compose file
rm $TEMP_COMPOSE_FILE

echo "Pre-init script complete."

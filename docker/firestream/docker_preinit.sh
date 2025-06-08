#!/bin/bash
set -e

# Enable debug mode with environment variable
if [ "${DEBUG:-false}" = "true" ]; then
    set -x
fi

# Get script directory and project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"


echo "SCRIPT_DIR: $SCRIPT_DIR"
echo "PROJECT_ROOT: $PROJECT_ROOT"
echo "Looking for files at:"
echo " - ${PROJECT_ROOT}/etc/.env.example"

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

detect_kvm() {
    if [ "$os" = "Linux" ] && [ -e "/dev/kvm" ]; then
        # Check if user has access to KVM
        if [ -r "/dev/kvm" ] && [ -w "/dev/kvm" ]; then
            echo "true"
        else
            echo "false"
        fi
    else
        echo "false"
    fi
}

# Add this line after HOST_GPU_VENDOR assignment
HOST_KVM_STATUS=$(detect_kvm)
echo "KVM Status: $HOST_KVM_STATUS"

get_host_ip() {
    case "$os" in
        Mac)
            # macOS-specific IP detection
            HOST_IP=$(ifconfig | grep "inet " | grep -v 127.0.0.1 | awk '{print $2}' | head -n1)
            ;;
        Linux)
            # Try ip command first (most modern Linux)
            if command -v ip >/dev/null 2>&1; then
                HOST_IP=$(ip -4 addr show scope global | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -n1)
            # Try hostname -I (some Linux distributions)
            elif command -v hostname >/dev/null 2>&1; then
                HOST_IP=$(hostname -I | awk '{print $1}')
            # Fallback to ifconfig
            elif command -v ifconfig >/dev/null 2>&1; then
                HOST_IP=$(ifconfig | grep -Eo 'inet (addr:)?([0-9]*\.){3}[0-9]*' | grep -Eo '([0-9]*\.){3}[0-9]*' | grep -v '127.0.0.1' | head -n1)
            fi
            ;;
    esac

    if [ -z "$HOST_IP" ]; then
        echo "Failed to get Host IP."
        return 1
    fi

    echo "$HOST_IP"
    return 0
}

# Use the function
HOST_IP=$(get_host_ip)
if [ $? -ne 0 ]; then
    echo "Warning: Failed to get Host IP, continuing anyway..."
fi

# Set the host user's username, user ID, and group ID
HOST_USERNAME="firestream"
HOST_USER_ID=1000
HOST_GROUP_ID=1000

# Extract GPU vendor for Linux
# Extract GPU vendor for Linux
extract_linux_gpu_vendor() {
    if command -v lspci &> /dev/null; then
        # If lspci is available, check for NVIDIA
        if lspci | grep -E "VGA|3D" | grep -iq "nvidia"; then
            echo "NVIDIA"
        else
            echo "Other"
        fi
    else
        # If lspci is not available, try alternative methods
        if [ -d "/proc/driver/nvidia" ] || [ -d "/usr/local/cuda" ]; then
            echo "NVIDIA"
        else
            # Default to "Other" if we can't detect GPU
            echo "Other"
        fi
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

# Function to set variable based on file existence
set_env_variable() {
    local example_file_path="${PROJECT_ROOT}/etc/.env.example"
    local expected_file_path="${PROJECT_ROOT}/etc/.env"

    if [ -e "$expected_file_path" ]; then
        ENV_SECRETS_PATH="$expected_file_path"
    elif [ -e "$example_file_path" ]; then
        ENV_SECRETS_PATH="$example_file_path"
    else
        echo "Error: Neither .env nor .env.example found"
        exit 1
    fi

    export ENV_SECRETS_PATH
    echo "ENV_SECRETS_PATH is set to: $ENV_SECRETS_PATH"
}

# Set up environment variables
set_env_variable

# Define file paths
BASE_FILE="${PROJECT_ROOT}/docker/firestream/docker-compose.yml"
TEMP_COMPOSE_FILE="${PROJECT_ROOT}/docker/firestream/docker-compose.temp.yml"


# Check if base compose file exists
if [ ! -f "$BASE_FILE" ]; then
    echo "Error: Base docker-compose.yml not found at: $BASE_FILE"
    exit 1
fi

# Create a temporary docker-compose file with environment variables and build arguments
{
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
    echo "      HOST_KVM_STATUS: $HOST_KVM_STATUS"
    if [ "$HOST_KVM_STATUS" = "true" ]; then
        echo "    devices:"
        echo "      - /dev/kvm:/dev/kvm"
    fi
    echo "    build:"
    echo "      args:"
    echo "        HOST_USER_ID: $HOST_USER_ID"
    echo "        HOST_GROUP_ID: $HOST_GROUP_ID"
    echo "        HOST_GPU_STATUS: $HOST_GPU_STATUS"
    echo "        HOST_GPU_VENDOR: $HOST_GPU_VENDOR"
    echo "        HOST_USERNAME: $HOST_USERNAME"
    echo "    env_file:"
    echo "      - $ENV_SECRETS_PATH"
} > "$TEMP_COMPOSE_FILE"

# Combine Compose Files based on GPU_STATUS
if [ "$HOST_GPU_VENDOR" = "NVIDIA" ]; then
    OVERRIDE_FILE="${PROJECT_ROOT}/docker/firestream/docker-compose.gpu_nvidia.yml"
    if [ ! -f "$OVERRIDE_FILE" ]; then
        echo "Error: NVIDIA override file not found at: $OVERRIDE_FILE"
        exit 1
    fi
    if ! docker compose -f "$BASE_FILE" -f "$OVERRIDE_FILE" -f "$TEMP_COMPOSE_FILE" config > "${PROJECT_ROOT}/docker/docker-compose.devcontainer.yml"; then
        echo "Error: Failed to generate docker-compose.devcontainer.yml with NVIDIA config"
        exit 1
    fi
else
    if ! docker compose -f "$BASE_FILE" -f "$TEMP_COMPOSE_FILE" config > "${PROJECT_ROOT}/docker/firestream/docker-compose.devcontainer.yml"; then
        echo "Error: Failed to generate docker-compose.devcontainer.yml"
        exit 1
    fi
fi

# Remove the temporary docker-compose file
rm "$TEMP_COMPOSE_FILE"

echo "Pre-init script complete."

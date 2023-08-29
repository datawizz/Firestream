#!/bin/bash
set -e

# Assert that Docker is available on the host
if ! command -v docker &> /dev/null; then
    echo "Docker must be installed to run this script."
    exit 1
fi

# Save and export the Docker Group ID of the host for use in the container
if [ "$(uname)" == "Darwin" ]; then
    # If on MacOS use the current user's GID
    export HOST_DOCKER_GID=$(id -g)
else
    export HOST_DOCKER_GID=$(getent group docker | cut -d: -f3)
fi
if [ -z "$HOST_DOCKER_GID" ]; then
    echo "Failed to get Docker Group ID."
    exit 1
fi


# Save and export the Machine ID of the host
if [ "$(uname)" == "Darwin" ]; then
    export HOST_MACHINE_ID=$(ioreg -rd1 -c IOPlatformExpertDevice | awk -F'"' '/IOPlatformUUID/{print $4}')
else
    export HOST_MACHINE_ID=$(cat /etc/machine-id)
fi
if [ -z "$HOST_MACHINE_ID" ]; then
    echo "Failed to get Machine ID."
    exit 1
fi

# Save and export the IP address of the host (used for end-to-end integration testing)
if [ "$(uname)" == "Darwin" ]; then
    export HOST_IP=$(ifconfig | grep "inet " | grep -Fv 127.0.0.1 | awk '{print $2}' | head -n 1)
else
    export HOST_IP=$(hostname -I | awk '{print $1}')
fi
if [ -z "$HOST_IP" ]; then
    echo "Failed to get Host IP."
    exit 1
fi

# Get the logged in user ID
export HOST_USER_ID=$(id -u)
if [ -z "$HOST_USER_ID" ]; then
    echo "Failed to get User ID."
    exit 1
fi

# Get the logged in user GID
export HOST_GROUP_ID=$(id -g)
if [ -z "$HOST_GROUP_ID" ]; then
    echo "Failed to get Group ID."
    exit 1
fi

# Function to get CPU architecture
get_cpu_architecture() {
  if [ "$(uname)" == "Darwin" ]; then
    echo "$(uname -m)"
  else
    echo "$(uname -m)"
  fi
}

# Function to get OS release name
get_os_release() {
  if [ "$(uname)" == "Darwin" ]; then
    echo "$(sw_vers -productName) $(sw_vers -productVersion)"
  else
    echo "$(lsb_release -d | cut -f2-)"
  fi
}

# Save and export the CPU architecture
export HOST_CPU_ARCH=$(get_cpu_architecture)
if [ -z "$HOST_CPU_ARCH" ]; then
  echo "Failed to get CPU architecture."
  exit 1
fi

# Save and export the OS release name
export HOST_OS_RELEASE=$(get_os_release)
if [ -z "$HOST_OS_RELEASE" ]; then
  echo "Failed to get OS release name."
  exit 1
fi

# Function to get GPU
get_gpu() {
  if [ "$(uname)" == "Darwin" ]; then
    echo "Apple"
  elif lspci | grep -i nvidia &> /dev/null; then
    echo "Nvidia"
  else
    echo "false"
  fi
}

# Save and export the GPU
export HOST_GPU=$(get_gpu)
if [ -z "$HOST_GPU" ]; then
  echo "Failed to get GPU."
  exit 1
fi





# Create a temporary docker-compose file
TEMP_COMPOSE_FILE="docker/docker-compose.temp.yml"
echo "services:" > $TEMP_COMPOSE_FILE
echo "  fireworks_devcontainer:" >> $TEMP_COMPOSE_FILE
echo "    environment:" >> $TEMP_COMPOSE_FILE
echo "      HOST_USER_ID: $HOST_USER_ID" >> $TEMP_COMPOSE_FILE
echo "      HOST_GROUP_ID: $HOST_GROUP_ID" >> $TEMP_COMPOSE_FILE
echo "      HOST_DOCKER_GID: $HOST_DOCKER_GID" >> $TEMP_COMPOSE_FILE
echo "      HOST_MACHINE_ID: $HOST_MACHINE_ID" >> $TEMP_COMPOSE_FILE
echo "      HOST_IP: $HOST_IP" >> $TEMP_COMPOSE_FILE
echo "      HOST_GPU_STATUS: $HOST_GPU_STATUS" >> $TEMP_COMPOSE_FILE
echo "      HOST_CPU_ARCH: $HOST_CPU_ARCH" >> $TEMP_COMPOSE_FILE
echo "      HOST_OS_RELEASE: $HOST_OS_RELEASE" >> $TEMP_COMPOSE_FILE


# Combine Compose Files
BASE_FILE="docker/docker-compose.yml"
if [ "$GPU_STATUS" = "true" ]; then
    OVERRIDE_FILE="docker/docker-compose.gpu_override.yml"
    docker compose -f $BASE_FILE -f $OVERRIDE_FILE -f $TEMP_COMPOSE_FILE config > docker/docker-compose.deployment.yml
else
    if [ "$DEPLOYMENT_MODE" = "test" ] || [ "$DEPLOYMENT_MODE" = "codespaces" ]; then
        OVERRIDE_FILE="docker/docker-compose.$DEPLOYMENT_MODE.yml"
        docker compose -f $BASE_FILE -f $OVERRIDE_FILE -f $TEMP_COMPOSE_FILE config > docker/docker-compose.deployment.yml
    else
        docker compose -f $BASE_FILE -f $TEMP_COMPOSE_FILE config > docker/docker-compose.deployment.yml
    fi
fi

# Remove the temporary docker-compose file
rm $TEMP_COMPOSE_FILE

echo "Pre-init script complete."

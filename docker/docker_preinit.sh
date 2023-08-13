#!/bin/bash

# Assert that Docker is available on the host
if ! command -v docker &> /dev/null; then
    echo "Docker must be installed to run this script."
    exit 1
fi

# Save and export the Docker Group ID of the host for use in the container
export HOST_DOCKER_GID=$(getent group docker | cut -d: -f3)
if [ -z "$HOST_DOCKER_GID" ]; then
    echo "Failed to get Docker Group ID."
    exit 1
fi

# Save and export the Machine ID of the host
export HOST_MACHINE_ID=$(cat /etc/machine-id)
if [ -z "$HOST_MACHINE_ID" ]; then
    echo "Failed to get Machine ID."
    exit 1
fi

# Save and export the IP address of the host (used for end-to-end integration testing)
export HOST_IP=$(hostname -I | awk '{print $1}')
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


# Save and export the GPU status
export HOST_GPU_STATUS="false"
if lspci | grep -i nvidia &> /dev/null; then
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

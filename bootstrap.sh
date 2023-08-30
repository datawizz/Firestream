#!/bin/bash
set -e

if [ "$#" -eq 1 ]; then
  export DEPLOYMENT_MODE="$1"
fi

echo "                                                                     "
echo "  Welcome to...                                                      "
echo "                                                                     "
echo "                                                                     "
echo "       _____.__                                   __                 "
echo "     _/ ____\__|______   ______  _  _____________|  | __  ______     "
echo "     \   __\|  \_  __ \_/ __ \ \/ \/ /  _ \_  __ \  |/ / /  ___/     "
echo "      |  |  |  ||  | \/\  ___/\     (  <_> )  | \/    <  \___ \      "
echo "      |__|  |__||__|    \___  >\/\_/ \____/|__|  |__|_ \/____  >     "
echo "                            \/                        \/     \/      "
echo "                                                                     "
echo "                                                                     "
echo "                                                                     "
echo "                    Starting Fireworks in < $DEPLOYMENT_MODE > mode  "
echo "                                                                     "


###############################################################################
### 0. Git Clone                                                            ###
###############################################################################

# Fetch submodules
# git submodule update --init --recursive

###############################################################################
### 1. Environment                                                          ###
###############################################################################

# Project Directory
_SRC="$(pwd)"

# Set the Machine ID on Debian host
export MACHINE_ID=${MACHINE_ID:-$(cat /var/lib/dbus/machine-id)}

# Ensure the deployment mode is one of the expected values
check_valid_mode() {
  local DEPLOYMENT_MODE=$1
  local valid=false
  local valid_modes=("development" "test" "clean" "cicd" "production" "resume" "interactive")
  
  for mode in "${valid_modes[@]}"; do
    if [ "$DEPLOYMENT_MODE" = "$mode" ]; then
      valid=true
      break
    fi
  done

  if [ "$valid" = false ]; then
    echo "Invalid DEPLOYMENT_MODE. It must be one of: ${valid_modes[*]}"
    exit 1
  fi
}

check_valid_mode $DEPLOYMENT_MODE

reboot_in_container() {
  local cmd="${1:-/bin/bash}"
  if [ -f /.dockerenv ]; then
    echo "Running inside the devcontainer."
  else
    echo "Not running in Docker container. Starting Initialization."
    bash docker/docker_preinit.sh
    docker compose -f docker/docker-compose.deployment.yml down --remove-orphans
    docker compose -f docker/docker-compose.deployment.yml build
    docker compose -f docker/docker-compose.deployment.yml run fireworks_devcontainer "$cmd"
  fi
}


### Code Version ###

# Set the commit hash
# This is used for container tagging and code versioning
if [[ $GIT_COMMIT_HASH != "latest" && $GIT_COMMIT_HASH != "development" ]]; then
    export GIT_COMMIT_HASH=$(git rev-parse --short HEAD)
fi

### Docker ###

# function to check if docker is available, offer to install it
function check_docker {
    if ! command -v docker &> /dev/null
    then
        echo "Docker is not available in the terminal"
        read -p "Do you want to install Docker? [y/N]: " user_input
        if [[ $user_input == 'y' || $user_input == 'Y' ]]
        then
            bash bin/host_scripts/host-docker.sh
        else
            exit 1
        fi
    else
        export DOCKER_VERSION=$(docker --version)
        echo "Docker version $DOCKER_VERSION is installed"
    fi
}

# function to check if the user has access to /var/run/docker.sock
function check_socket {
    local docker_socket="/var/run/docker.sock"

    if [ ! -e $docker_socket ]
    then
        echo "$docker_socket does not exist"
        exit 1
    elif [ ! -r $docker_socket ] || [ ! -w $docker_socket ]
    then
        echo "The user does not have the required read and write permissions on $docker_socket"
        exit 1
    else
        echo "The Docker socket is available for R/W at $docker_socket"
    fi
}

# run the checks if DEPLOYMENT_MODE is "development" or "test"
if [ "$DEPLOYMENT_MODE" == "development" ] || [ "$DEPLOYMENT_MODE" == "test" ] || [ "$DEPLOYMENT_MODE" == "interactive" ]; then
    check_docker
    check_socket
fi


# Check CPU architecture and save it to a variable
cpu_arch=$(uname -m)
export CPU_ARCHITECTURE=$cpu_arch
echo "CPU Architecture: $cpu_arch"

# CPU resources
total_cpu=$(nproc)
export TOTAL_CPU_RESOURCES=$total_cpu
echo "Total CPU cores (including Hyperthreads): $TOTAL_CPU_RESOURCES"

# Memory resources
total_memory=$(free -m | awk '/^Mem:/{print $2}')
export TOTAL_MEMORY_RESOURCES=$total_memory
echo "Total Memory resources (in MB): $TOTAL_MEMORY_RESOURCES"

# Check for NVIDIA GPU
if lspci | grep -i nvidia > /dev/null; then
    echo "NVIDIA GPU detected"
    export HAS_NVIDIA_GPU=true
    # Install the Nvidia container tools
    sudo /bin/bash bin/host_scripts/nvidia-debian.sh
else
    echo "No NVIDIA GPU detected"
    export HAS_NVIDIA_GPU=false
fi

echo "Environment successfully configured"

###############################################################################
### 1. Configure Host                                                       ###
###############################################################################
# If this script is being run on a Debian OS directly, ensure the host is configured with the required pakcages
# If this scirpt is being run in a container, assume the host is configured correctly.

#TODO this needs to be run once on new host configuration
# /home/fireworks/Fireworks/bin/host_scripts/max_memory_map.sh

###############################################################################
### 2. Deployment                                                           ###
###############################################################################


# 1. If in development mode
if [ "$DEPLOYMENT_MODE" = "development" ]; then

  reboot_in_container bash bootstrap.sh development

  # Setup a K3D cluster on the host's Docker Engine and
  # route the devcontainer's DNS to the K8 Control Plane for internal DNS resolution
  bash $_SRC/docker/k3d/bootstrap.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Install source python packages in editable mode
  bash $_SRC/bin/cicd_scripts/bootstrap_devcontainer.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Build the project's container images and artifacts
  #bash /workspace/bin/cicd_scripts/build.sh
  #TODO include builds inline

  # Helm Install Charts
  bash $_SRC/bin/cicd_scripts/helm_install.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Run port forwarding for the services to localhost
  bash $_SRC/bin/cicd_scripts/port_forward.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Set S3 Credentials
  bash $_SRC/bin/commands/set_s3_credentials.sh
  if [ $? -ne 0 ]; then exit 1; fi
fi


# 2. If in test mode
if [ "$DEPLOYMENT_MODE" = "test" ]; then
  # TODO make the cluster name random

  # Ensure the process is running in a container
  reboot_in_container make test

  # Ensure the development cluster is running
  bash bootstrap.sh development

  # Run the tests
  pytest
fi

# 2.1. If in resume mode
if [ "$DEPLOYMENT_MODE" = "resume" ]; then
  # Setup a K3D cluster on the host's Docker Engine and
  # route the devcontainer's DNS to the K8 Control Plane for internal DNS resolution
  bash $_SRC/docker/k3d/bootstrap.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Install source python packages in editable mode
  bash $_SRC/bin/cicd_scripts/bootstrap_devcontainer.sh
  if [ $? -ne 0 ]; then exit 1; fi
fi


# 3. If in "clean" mode then just create the cluster but don't install services
if [ "$DEPLOYMENT_MODE" = "clean" ]; then
  # Setup a K3D cluster on the host's Docker Engine and
  # route the devcontainer's DNS to the K8 Control Plane for internal DNS resolution
  bash $_SRC/docker/k3d/bootstrap.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Install source python packages in editable mode
  bash $_SRC/bin/cicd_scripts/bootstrap_devcontainer.sh
  if [ $? -ne 0 ]; then exit 1; fi

fi


# 3. If in production mode
if [ "$DEPLOYMENT_MODE" = "production" ]; then
  
  #TODO

  # 3.1   Build the project's container images and artifacts
  # 3.2   Push the images to the remote container registry
  # 3.3   Deploy the project's services using Helm on the remote cluster
  # 3.4   ???
  # 3.5   Profit
  echo "TODO"
  exit 1
fi


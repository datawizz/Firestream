#!/bin/bash
set -e

if [ -z "${DEPLOYMENT_MODE}" ] || [ -z "$(eval echo \$$DEPLOYMENT_MODE)" ]; then
    DEPLOYMENT_MODE_MESSAGE="bare_metal"
else
    DEPLOYMENT_MODE_MESSAGE=$(eval echo \$$DEPLOYMENT_MODE)
fi

echo "                                                                                  "
echo "  Welcome to...                                                                   "
echo "                                                                                  "
echo "                                                                                  "
echo "   ███████ ██ ██████  ███████ ███████ ████████ ██████  ███████  █████  ███    ███ "
echo "   ██      ██ ██   ██ ██      ██         ██    ██   ██ ██      ██   ██ ████  ████ "
echo "   █████   ██ ██████  █████   ███████    ██    ██████  █████   ███████ ██ ████ ██ "
echo "   ██      ██ ██   ██ ██           ██    ██    ██   ██ ██      ██   ██ ██  ██  ██ "
echo "   ██      ██ ██   ██ ███████ ███████    ██    ██   ██ ███████ ██   ██ ██      ██ "
echo "                                                                                  "
echo "                                                                                  "
echo "                                                                                  "
echo "               Starting FireStream in < $DEPLOYMENT_MODE_MESSAGE > mode            "
echo "                                                                                  "

                                                                              
                                                                               

###############################################################################
### 0. Git Clone                                                            ###
###############################################################################

# Fetch submodules
# git submodule update --init --recursive

###############################################################################
### 1. Environment                                                          ###
###############################################################################

# Project Directory
_SRC="/workspace"

# Set the Machine ID on Debian host
export MACHINE_ID=${MACHINE_ID:-$(cat /var/lib/dbus/machine-id)}

# Define valid deployment modes
valid_modes=("development" "test" "clean" "cicd" "production" "resume" "build")

# Check if an argument is provided
if [ $# -eq 0 ]; then
    echo "No argument provided. Exiting."
    exit 1
fi

# Set the first argument as the DEPLOYMENT_MODE environment variable
export DEPLOYMENT_MODE="$1"

# Check if the argument is valid
valid=false
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

### Code Version ###

# Set the commit hash
# This is used for container tagging and code versioning
if [[ $GIT_COMMIT_HASH != "latest" && $GIT_COMMIT_HASH != "development" ]]; then
    export GIT_COMMIT_HASH=$(git rev-parse --short HEAD)
fi

### Docker ###

# function to check if docker is available
function check_docker {
    if ! command -v docker &> /dev/null
    then
        echo "Docker is not available in the terminal"
        exit 1
    else
        echo "Docker version $(docker --version) is installed"
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

# run the checks if DEPLOYMENT_MODE is "development" or "build"
if [ "$DEPLOYMENT_MODE" == "development" ] || [ "$DEPLOYMENT_MODE" == "build" ] || [ "$DEPLOYMENT_MODE" == "test" ]; then
    check_docker
    check_socket
fi




# Check if running inside Docker
if [ -f /.dockerenv ]; then
    echo "Running inside Docker, will not start Docker Compose."
elif grep -qa docker /proc/1/cgroup; then
    echo "Running inside Docker, will not start Docker Compose."
elif [ -e /proc/self/cgroup ] && (grep -qE '/docker/' /proc/self/cgroup || grep -qE '/docker-ce/' /proc/self/cgroup); then
    echo "Running inside Docker, will not start Docker Compose."
else
    echo "Not running inside Docker, attempting to start Docker Compose."
    # Run Docker Compose command
    docker compose -f docker/docker-compose.test.yml run devcontainer bash -c "/workspace/bootstrap.sh ${DEPLOYMENT_MODE}"
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
# /home/firestream/FireStream/bin/host_scripts/max_memory_map.sh

###############################################################################
### 2. Deployment                                                           ###
###############################################################################

# 1. If in test mode
if [ "$DEPLOYMENT_MODE" = "test" ]; then
	bash $_SRC/bin/cicd_scripts/test.sh
fi

# 1. If in build mode
if [ "$DEPLOYMENT_MODE" = "build" ]; then
	bash $_SRC/bin/cicd_scripts/build.sh
fi

# 2. If in development mode
if [ "$DEPLOYMENT_MODE" = "development" ]; then

  # Setup a K3D cluster on the host's Docker Engine and
  # route the devcontainer's DNS to the K8 Control Plane for internal DNS resolution
  bash $_SRC/docker/k3d/bootstrap.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Install source python packages in editable mode
  bash $_SRC/bin/cicd_scripts/bootstrap_devcontainer.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Build the project's container images and artifacts
  #bash /workspace/bin/cicd_scripts/build.sh

  # Helm Install Charts
  bash $_SRC/bin/cicd_scripts/helm_install.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Run port forwarding for the services to localhost
  bash $_SRC/bin/cicd_scripts/port_forward.sh
  if [ $? -ne 0 ]; then exit 1; fi
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


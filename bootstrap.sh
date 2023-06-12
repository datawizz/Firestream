#!/bin/bash
set -e

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

## Clone the project and the submodules that it depends on
# TODO
# git clone --recurse-submodules https://github.com/datawizz/fireworks


###############################################################################
### 1. Environment                                                          ###
###############################################################################

# Define valid deployment modes
valid_modes=("development" "test" "clean" "cicd" "production")

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

# run the checks if DEPLOYMENT_MODE is "development" or "cicd"
if [ "$DEPLOYMENT_MODE" == "development" ] || [ "$DEPLOYMENT_MODE" == "cicd" ]; then
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
else
    echo "No NVIDIA GPU detected"
    export HAS_NVIDIA_GPU=false
fi

echo "Environment successfully configured"

###############################################################################
### 2. Deployment                                                           ###
###############################################################################

# 1. If in test mode
if [ "$DEPLOYMENT_MODE" = "test" ]; then
  # 1.1   Run test suite via Docker Compose for each service
  # 1.2   Run the integration suite via Helm and Docker Compose
  echo "TODO"
fi

# 2. If in development mode
if [ "$DEPLOYMENT_MODE" = "development" ]; then

  # Install source python packages in editable mode
  bash /workspace/bin/cicd_scripts/bootstrap_devcontainer.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Setup a K3D cluster on the host's Docker Engine and
  # route the devcontainer's DNS to the K8 Control Plane for internal DNS resolution
  bash /workspace/docker/k3d/bootstrap.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Build the project's container images and artifacts
  #bash /workspace/bin/cicd_scripts/build.sh

  # Helm Install Charts
  bash /workspace/bin/cicd_scripts/helm_install.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Run port forwarding for the services to localhost
  bash /workspace/bin/cicd_scripts/port_forward.sh
  if [ $? -ne 0 ]; then exit 1; fi
fi

# 3. If in "clean" mode then just create the cluster but don't install services
if [ "$DEPLOYMENT_MODE" = "clean" ]; then

  # Install source python packages in editable mode
  bash /workspace/bin/cicd_scripts/bootstrap_devcontainer.sh
  if [ $? -ne 0 ]; then exit 1; fi

  # Setup a K3D cluster on the host's Docker Engine and
  # route the devcontainer's DNS to the K8 Control Plane for internal DNS resolution
  bash /workspace/docker/k3d/bootstrap.sh
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


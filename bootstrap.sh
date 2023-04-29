#!/bin/bash
echo ""
echo ""
echo "       _____.__                                   __                 "
echo "     _/ ____\__|______   ______  _  _____________|  | __  ______     "
echo "     \   __\|  \_  __ \_/ __ \ \/ \/ /  _ \_  __ \  |/ / /  ___/     "
echo "      |  |  |  ||  | \/\  ___/\     (  <_> )  | \/    <  \___ \      "
echo "      |__|  |__||__|    \___  >\/\_/ \____/|__|  |__|_ \/____  >     "
echo "                            \/                        \/     \/      "
echo ""
echo ""

set -e




###############################################################################
### 0. Git Clone                                                            ###
###############################################################################

## Ensure that submodules are cloned
# TODO
# git clone --recurse-submodules https://github.com/datawizz/fireworks

####
# Scope the resources avaialble to the project.
#TODO 

# Assert that there is sufficent CPU power for the project's services

# echo /proc/cpuinfo | grep -m 1 "cpu cores"
# cat /proc/meminfo | grep -m 1 "MemTotal"

###############################################################################
### 1. Environment                                                          ###
###############################################################################

# Set the commit hash for tagging the container images
export GIT_COMMIT_HASH=$(git rev-parse --short HEAD)

#TODO check if docker is available



# Check CPU architecture and save it to a variable
cpu_arch=$(uname -m)
echo "CPU Architecture: $cpu_arch"

# Check for Nvidia GPU presence and save it to a variable
nvidia_gpu="Not Found"
if lspci | grep -q "NVIDIA"; then
    nvidia_gpu="Found"
fi
echo "NVIDIA GPU: $nvidia_gpu"

# The deployment mode must be provided either in a argument at runtime or as a environment variable of the same name
deployment_mode="${1:-$DEPLOYMENT_MODE}"

# Check if the argument is valid
if [ "$deployment_mode" != "development" ] && [ "$deployment_mode" != "test" ]; then
  echo "Invalid deployment_mode. It must be either 'development' or 'test'"
  exit 1
fi

echo "Bootstraping Fireworks in < $deployment_mode > mode"


###############################################################################
### 2. Deployment                                                           ###
###############################################################################

# 1. If in test mode
if [ "$deployment_mode" = "test" ]; then
  # 1.1   Run test suite via Docker Compose for each service
  # 1.2   Run the integration suite via Helm and Docker Compose
  echo "TODO"
fi

# 2. If in development mode
if [ "$deployment_mode" = "development" ]; then

  # Setup a K3D cluster on the host's Docker Engine and
  # route the devcontainer's DNS to the K8 Control Plane for internal DNS resolution
  sh /workspace/docker/k3d/bootstrap.sh

  # Build the project's container images and artifacts
  #TODO make idempotent 
  # sh /workspace/bin/cicd_scripts/build.sh

  # Install source python packages in editable mode
  sh /workspace/bin/cicd_scripts/bootstrap_devcontainer.sh
fi

# 3. If in production mode
if [ "$deployment_mode" = "production" ]; then
  
  #TODO

  # 3.1   Build the project's container images and artifacts
  # 3.2   Push the images to the container registry
  # 3.3   Deploy the project's services using Helm
  # 3.4   ???
  # 3.5   Profit
  echo "TODO"
fi



###############################################################################
### 3. Deployment                                                           ###
###############################################################################

# Build Artifacts
#sh /workspace/opt/cicd_scripts/build.sh

# Helm Install Charts
sh /workspace/bin/cicd_scripts/helm_install.sh

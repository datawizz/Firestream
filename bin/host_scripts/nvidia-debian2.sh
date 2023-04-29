


# For GPU Training (which might be moved to another repo?) there is a requirement that the docker daemon be launched with an extension
# This is part of the nvidia container toolkit

# Assumptions
# The host OS is Ubuntu 20.04.
# The host has at least one nvidia GPU w/ CUDA

# 1. Ensure that this is Ubuntu 20.04
# 2. Download the latest nvidia drivers and install
# 3. Download Docker for the host system
# 4. Download the latest nvidia container runtime
# 5. Run a GPU accelerated container image for a smoke test

# Add user group to docker
sudo usermod -aG docker $USER

# Install Nvidia Container Runtime
# https://nvidia.github.io/nvidia-container-runtime/
curl -s -L https://nvidia.github.io/nvidia-container-runtime/gpgkey | \
  sudo apt-key add -
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-container-runtime/$distribution/nvidia-container-runtime.list | \
  sudo tee /etc/apt/sources.list.d/nvidia-container-runtime.list
sudo apt-get update && apt-get install nvidia-container-runtime

# Ensure docker is not running for this
sudo dockerd --add-runtime=nvidia=/usr/bin/nvidia-container-runtime



# Start docker and check that it is installed
sudo service docker start


docker info|grep -i runtime


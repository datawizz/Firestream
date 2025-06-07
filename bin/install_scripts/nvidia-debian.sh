#!/bin/bash

# This script searches for NVIDIA drivers, finds the latest one, and installs it.

# Check if the script is run as sudo
if [ "$(id -u)" != "0" ]; then
    echo "This script must be run as root" 1>&2
    exit 1
fi

# Check for NVIDIA GPU
lspci | grep -i nvidia > /dev/null
if [ $? -eq 0 ]; then
    echo "NVIDIA GPU detected."
else
    echo "No NVIDIA GPU detected. Exiting."
    exit 1
fi

# Use noninteractive frontend for debconf to avoid prompts.
export DEBIAN_FRONTEND=noninteractive
export DEBCONF_NONINTERACTIVE_SEEN=true

# Update and install prerequisites
echo "Updating system and installing prerequisites..."
apt update -y && apt upgrade -y && apt install -y curl gnupg2 software-properties-commo


# Add NVIDIA package repositories
echo "Adding NVIDIA package repositories..."
curl -sL https://nvidia.github.io/nvidia-docker/gpgkey | apt-key add -
curl -sL https://nvidia.github.io/nvidia-docker/ubuntu$(lsb_release -rs)/nvidia-docker.list | tee /etc/apt/sources.list.d/nvidia-docker.list


# Step 1: Find all NVIDIA drivers.
# The output of apt search is piped into grep, which extracts lines containing "nvidia-" followed by one or more digits.
# The '-P' flag tells grep to use Perl-compatible regular expressions, which are more powerful and flexible than traditional regular expressions.
# The '-o' flag tells grep to output only the part of the line that matches the regular expression.
# The regular expression 'nvidia-\K\d+' matches the string "nvidia-" followed by one or more digits, but because of the '\K', only the digits are output.
drivers=$(apt search nvidia-driver | grep -Po 'nvidia-\K\d+')

# Step 2: Find the latest NVIDIA driver.
# The output of the previous command is piped into sort, which sorts its input.
# The '-nr' flags tell sort to sort numerically (so 100 comes after 20) and in reverse (so larger numbers come first).
# The 'head -n 1' command then takes the first line of its input, which is the highest version number.
latest_driver=$(echo "$drivers" | sort -nr | head -n 1)


# Step 3: Install the latest NVIDIA driver.
# The 'sudo apt-get install -y' command automatically installs the given package without prompting for confirmation.
# The "xserver-xorg-video-nvidia-$latest_driver" is the name of the package to install.
apt-get install -y --allow-change-held-packages "xserver-xorg-video-nvidia-$latest_driver"

# apt install xserver-xorg-video-nvidia-450-server # CUDA 11 baby!

# After the installation, use needrestart with "-r a" to automatically restart all services that need it.
needrestart -r a

# Test NVIDIA driver installation
echo "Testing NVIDIA driver installation..."
nvidia-smi
if [ $? -eq 0 ]; then
    echo "NVIDIA driver installed successfully."
else
    echo "NVIDIA driver installation failed. Exiting."
    exit 1
fi


### NVIDIA containerd runtime ###

echo "Installing NVIDIA Container Toolkit..."
apt install -y nvidia-container-toolkit
# Restart all services which require it
needrestart -r a

nvidia-ctk runtime configure --runtime=docker --set-as-default

systemctl daemon-reload \
  && systemctl restart docker

needrestart -r a

DOCKER_INFO=$(docker info)

echo "${DOCKER_INFO}" | grep -i 'runtime.*nvidia'

if [ $? -eq 0 ]; then
    echo "Nvidia runtime found."
else
    echo "Nvidia runtime not found. Exiting with failure status."
    exit 1
fi

# Set the default
nvidia-ctk runtime configure --runtime=docker --set-as-default



# Test NVIDIA Docker installation
echo "Testing NVIDIA Docker installation..."
docker run -it --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi
if [ $? -eq 0 ]; then
    echo "NVIDIA Docker installed successfully."
else
    echo "NVIDIA Docker installation failed. Exiting."
    exit 1
fi

echo "All components installed and tested successfully."

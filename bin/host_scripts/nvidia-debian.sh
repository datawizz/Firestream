#!/bin/bash

# Check for NVIDIA GPU
lspci | grep -i nvidia > /dev/null
if [ $? -eq 0 ]; then
    echo "NVIDIA GPU detected."
else
    echo "No NVIDIA GPU detected. Exiting."
    exit 1
fi

# Update and install prerequisites
echo "Updating system and installing prerequisites..."
sudo apt-get update
sudo apt-get install -y curl gnupg2 software-properties-common

# Add NVIDIA package repositories
echo "Adding NVIDIA package repositories..."
curl -sL https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -sL https://nvidia.github.io/nvidia-docker/ubuntu$(lsb_release -rs)/nvidia-docker.list | sudo tee /etc/apt/sources.list.d/nvidia-docker.list
sudo apt-get update

# Install NVIDIA Docker and latest driver
echo "Installing NVIDIA Docker and latest driver..."
sudo apt-get install -y nvidia-docker2 nvidia-driver-$(apt-cache search nvidia-driver-[0-9]* | awk '{print $1}' | sort -r | head -n 1)

# Restart Docker and load NVIDIA kernel module
sudo systemctl restart docker
sudo modprobe nvidia

# Test NVIDIA driver installation
echo "Testing NVIDIA driver installation..."
nvidia-smi
if [ $? -eq 0 ]; then
    echo "NVIDIA driver installed successfully."
else
    echo "NVIDIA driver installation failed. Exiting."
    exit 1
fi

# Test NVIDIA Docker installation
echo "Testing NVIDIA Docker installation..."
docker run --rm --gpus all nvidia/cuda:11.0-base nvidia-smi
if [ $? -eq 0 ]; then
    echo "NVIDIA Docker installed successfully."
else
    echo "NVIDIA Docker installation failed. Exiting."
    exit 1
fi

echo "All components installed and tested successfully."

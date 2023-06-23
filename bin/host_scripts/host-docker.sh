#!/bin/bash

# Install docker on Debian
# This script should be run directly on a clean Ubuntu host to install the required VM engine.

set -e

# Check if the script was run as root or through sudo
if [ "$EUID" -ne 0 ]; then 
  echo "Please run this script as sudo"
  exit
fi

# If the script was run with sudo, SUDO_USER will be set to the name of the user that invoked it
if [ -n "$SUDO_USER" ]; then
  echo "This script was run with sudo by user: $SUDO_USER"
else
  echo "This script was run by the root user"
fi



export DEBIAN_FRONTEND=noninteractive && apt update -y && apt -y install \
    ca-certificates \
    curl \
    gnupg \
    lsb-release





# Define the Docker GPG key and APT source list
DOCKER_GPG="/usr/share/keyrings/docker-archive-keyring.gpg"
DOCKER_APT_SOURCE="/etc/apt/sources.list.d/docker.list"

# Check for Docker GPG key and add if it doesn't exist
if [ ! -f "$DOCKER_GPG" ]; then
    echo -e "Y\n" | curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o "$DOCKER_GPG"
fi

# Check for Docker APT source list and add if it doesn't exist
if [ ! -f "$DOCKER_APT_SOURCE" ]; then
    echo "deb [arch=$(dpkg --print-architecture) signed-by=$DOCKER_GPG] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee "$DOCKER_APT_SOURCE" > /dev/null
fi


# Install Docker
export DEBIAN_FRONTEND=noninteractive && sudo apt-get update && sudo apt-get -y install docker-ce docker-ce-cli containerd.io docker-compose-plugin

# Check if docker group exists, if not create it
if ! getent group docker >/dev/null; then
    groupadd docker
fi

# Check if the user is in the docker group, if not add him/her
if ! id -nG "$SUDO_USER" | grep -qw docker; then
    usermod -aG docker $SUDO_USER
fi




# Modify permissions on docker socket
if [ -S /var/run/docker.sock ]; then
    chown :docker /var/run/docker.sock
    chmod 660 /var/run/docker.sock
fi

# Restart Docker service
if systemctl is-active --quiet docker; then
    systemctl restart docker
else
    systemctl start docker
fi

# Enable Docker services if not already enabled
if ! systemctl is-enabled --quiet docker.service; then
    systemctl enable docker.service
fi

if ! systemctl is-enabled --quiet containerd.service; then
    systemctl enable containerd.service
fi

# Check if docker service is running, if not, start it
if ! service --status-all | grep -Fq 'docker'; then
    service docker start
fi

# Enable service at Start Up if not already enabled
if ! ls /etc/rc*.d/*docker* > /dev/null 2>&1; then
    update-rc.d docker defaults
fi

# Test if docker is configured correctly
docker run hello-world

echo "Docker Installation Complete. The system needs to be rebooted for changes to take effect."

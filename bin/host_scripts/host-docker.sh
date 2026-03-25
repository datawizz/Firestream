#!/bin/bash

# Install Docker on Debian or Ubuntu
# This script should be run directly on a clean Debian or Ubuntu host to install the required Docker engine.

set -e

# Check if the script was run as root or through sudo
if [ "$EUID" -ne 0 ]; then 
  echo "Please run this script as sudo"
  exit 1
fi

# If the script was run with sudo, SUDO_USER will be set to the name of the user that invoked it
if [ -n "$SUDO_USER" ]; then
  echo "This script was run with sudo by user: $SUDO_USER"
else
  echo "This script was run by the root user"
fi

# Update package index and install necessary packages
export DEBIAN_FRONTEND=noninteractive && apt update -y && apt -y install \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

# Define the Docker GPG key and APT source list
DOCKER_GPG="/etc/apt/keyrings/docker.asc"
DOCKER_APT_SOURCE="/etc/apt/sources.list.d/docker.list"

# Create the keyrings directory if it doesn't exist
install -m 0755 -d /etc/apt/keyrings

# Add Docker's official GPG key
curl -fsSL https://download.docker.com/linux/$(lsb_release -is | tr '[:upper:]' '[:lower:]')/gpg -o "$DOCKER_GPG"
chmod a+r "$DOCKER_GPG"

# Add the Docker APT repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=$DOCKER_GPG] https://download.docker.com/linux/$(lsb_release -is | tr '[:upper:]' '[:lower:]') \
  $(lsb_release -cs) stable" | tee "$DOCKER_APT_SOURCE" > /dev/null

# Update package index again
apt-get update

# Install Docker packages
export DEBIAN_FRONTEND=noninteractive && apt-get -y install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Check if docker group exists, if not create it
if ! getent group docker >/dev/null; then
    groupadd docker
fi

# Check if the user is in the docker group, if not add them
if [ -n "$SUDO_USER" ]; then
    if ! id -nG "$SUDO_USER" | grep -qw docker; then
        usermod -aG docker $SUDO_USER
    fi
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


sudo apt install make
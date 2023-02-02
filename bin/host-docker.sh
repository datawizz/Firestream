#!/usr/bin/env bash

# Install docker on Debian
# This script should be run directly on a clean linux install

set -e

export DEBIAN_FRONTEND=noninteractive 

apt-get update && \
apt-get -y install \
    ca-certificates \
    curl \
    gnupg \
    lsb-release \
    acl


#TODO make this idempotent
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

echo \
  "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker
export DEBIAN_FRONTEND=noninteractive && apt-get update && apt-get -y install docker-ce docker-ce-cli containerd.io docker-compose-plugin

# Add user to docker permissions
usermod -a -G docker $USER
setfacl --modify user:$USER:rw /var/run/docker.sock

# Establish services
systemctl enable docker.service
systemctl enable containerd.service

# Start docker
service docker start
# Start service at Start Up
update-rc.d docker defaults


# Test if docker is configured correctly
docker run hello-world
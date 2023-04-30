# Install docker on Debian
# This script should be run directly on a clean Ubuntu host to install the required VM engine.

set -e

#TODO check that this is run as root

export DEBIAN_FRONTEND=noninteractive && apt update -y && apt -y install \
    ca-certificates \
    curl \
    gnupg \
    lsb-release


#TODO make this idempotent
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker
export DEBIAN_FRONTEND=noninteractive && sudo apt-get update && sudo apt-get -y install docker-ce docker-ce-cli containerd.io docker-compose-plugin

# Create docker group and add current user
sudo groupadd docker
sudo usermod -a -aG docker $USER

# Modify permissions on docker socket
sudo chown :docker /var/run/docker.sock
sudo chmod 660 /var/run/docker.sock

# Restart Docker service
systemctl restart docker

# Establish services
systemctl enable docker.service
systemctl enable containerd.service

# Start docker
service docker start
# Start service at Start Up
update-rc.d docker defaults


# Test if docker is configured correctly
# docker run hello-world


echo "Docker Installtation Complete. The system needs to be rebooted for changes to take effect.
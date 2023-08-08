
# Commands to run on the host before the devcontainer is created

# Assert that Docker is available on the host

# Save the Docker Group ID of the host for use in the container

# Save the Machine ID of the host

# Save the ip address of the host (used for end to end intergration testing)

# Launch devcontainer with GPU features
lspci | grep -i nvidia &>/dev/null && { cp docker/devcontainer_gpu.docker-compose.yml .devcontainer/docker-compose.yml; } || { cp docker/devcontainer.docker-compose.yml .devcontainer/docker-compose.yml; }
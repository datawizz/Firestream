#!/bin/bash

# The set -e option instructs bash to immediately exit if any command [1] has a non-zero exit status i.e. all tests must pass!
set -e

# Start Docker
echo "Starting Docker"
bash /usr/local/share/docker-init.sh
#Ensure it is started
docker --version

sleep infinity
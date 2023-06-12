#!/bin/bash

echo "WARNING! This command will reset the Fireworks environment by killing all Docker containers related to the project, and delete all images, networks, and volumes associated with them. Please reply Y to continue and N to exit"
read -p "Do you want to continue? (Y/N): " -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    # Stop and remove containers, volumes and networks associated with "fireworks"
    export DEBIAN_FRONTEND=noninteractive
    docker stop $(docker ps -a | grep "fireworks" | awk '{print $1}')
    docker rm $(docker ps -a | grep "fireworks" | awk '{print $1}')
    docker network prune -f
else
    exit 0
fi


echo "WARNING! This command will reset the Firestream environment by killing all Docker containers related to the project, and delete all images, networks, and volumes associated with them. Please reply Y to continue and N to exit"
read -p "Do you want to continue? (Y/N): " -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    # Stop and remove containers, volumes and networks associated with "firestream"
    export DEBIAN_FRONTEND=noninteractive

    # Stop and remove containers with "firestream" in their name
    docker ps -a --filter "name=firestream" -q | xargs docker stop
    docker ps -a --filter "name=firestream" -q | xargs docker rm

    # Remove images with "firestream" in their repository or tag
    docker images --filter "reference=*firestream*" -q | xargs docker rmi

    # Remove networks and volumes with "firestream" in their name
    docker network ls --filter "name=firestream" -q | xargs docker network rm
    docker volume ls --filter "name=firestream" -q | xargs docker volume rm

    # Clean up dangling resources
    docker system prune -f
else
    exit 0
fi
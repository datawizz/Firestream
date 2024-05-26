
echo "WARNING! This command will reset the Fireworks environment by killing all Docker containers related to the project, and delete all images, networks, and volumes associated with them. Please reply Y to continue and N to exit"
read -p "Do you want to continue? (Y/N): " -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    # Stop and remove containers, volumes and networks associated with "fireworks"
    export DEBIAN_FRONTEND=noninteractive

    # Stop and remove containers with "fireworks" in their name
    docker ps -a --filter "name=fireworks" -q | xargs docker stop
    docker ps -a --filter "name=fireworks" -q | xargs docker rm

    # Remove images with "fireworks" in their repository or tag
    docker images --filter "reference=*fireworks*" -q | xargs docker rmi

    # Remove networks and volumes with "fireworks" in their name
    docker network ls --filter "name=fireworks" -q | xargs docker network rm
    docker volume ls --filter "name=fireworks" -q | xargs docker volume rm

    # Clean up dangling resources
    docker system prune -f
else
    exit 0
fi
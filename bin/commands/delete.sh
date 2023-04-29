export DEBIAN_FRONTEND=noninteractive

# (optional) Force new builds and clean old installations of the platform
docker stop $(docker ps -a -q)
docker rm $(docker ps -a -q)
docker volume rm $(docker volume ls -q)
docker network prune -f
#sudo chmod -R 700 /home/USER

#!/usr/bin/env bash

if ! command -v 'docker' &> /dev/null
then
    echo "Error: docker not found, please install"
    exit 1
fi

if docker ps | grep -q 'local-mariadb'; then
    echo "Abort: local-mariadb container already running"
    exit 1
fi

#if docker ps -all | grep -q 'local-mariadb'; then
#    echo "Existing container found, but its stopped, removing"
#    docker remove local-mariadb
#fi

# Start local mariadb container
# --detach: Run container in background
# --name: Assign a name to the container
# --network: Use a internal network with filtered ports
# --restart: Restart the container unless it is explicitly stopped
# --log-driver: Log to a local file with a maximum size to avoid filling up the disk
# --env: Default root password for mariadb in the development environment
# -p: Expose port 3306 to allow access from host
# Use the latest mariadb image
docker run \
  --detach \
  --rm \
  --name local-mariadb \
  --network="bridge" \
  --log-driver local --log-opt max-size=10m \
  --env MARIADB_ROOT_PASSWORD=root \
  --env MARIADB_USER=pr \
  --env MARIADB_PASSWORD=pr \
  --env MARIADB_DATABASE=pocket_rocket \
  -p 3306:3306 \
  mariadb:latest

  # --restart=unless-stopped \

echo "Done"

#!/usr/bin/env bash

if ! command -v 'docker' &> /dev/null
then
    echo "Error: docker not found, please install"
    exit 1
fi

# -it for interactive
# --rm to remove container after exit
# local-mariadb name of the container
# mariadb:latest to use latest image
# Database credentials and settings for debug environmen
# docker run -it --rm --network="host" mariadb:latest \
#   mariadb --host 127.0.0.1 --port 3306 --user root --password=root
docker exec -it local-mariadb mariadb --user=root --password=root pocket_rocket


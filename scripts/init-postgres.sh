#!/bin/bash

#
# this script should be run from the repo root:
# example:
# $ ./scripts/init-postgres.sh

export VOLUME_NAME=${1:-financedb_data}

# allow the core postgres files to be owned by the user "postgres"
docker volume create $VOLUME_NAME

echo "Starting initial container to initialize core db directories"
docker run -d --rm --name financedb_core -v $VOLUME_NAME:/var/lib/postgresql/data -e POSTGRES_PASSWORD=postgres postgres:11

echo "Sleeping for 3 seconds"
sleep 3
docker rm -f financedb_core

# create the private user volume
docker run -it --rm -v $VOLUME_NAME:/var/lib/postgresql/data bash chown -R "$(id -u):$(id -g)" \
  /var/lib/postgresql/data

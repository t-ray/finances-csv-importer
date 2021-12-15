#!/bin/bash
#
# this script should be run from the repo root:
# example:
# $ ./scripts/run-postgres.sh

export VOLUME_NAME=${1:-financedb_data}
docker run -d \
    --name finance-db \
    --user "$(id -u):$(id -g)" \
    -v $VOLUME_NAME:/var/lib/postgresql/data \
    -p 15432:5432 \
    postgres:11

#!/usr/bin/env bash
# This script meant to be run on Unix/Linux based systems
set -e

echo "*** Start Anmol Substrate node ***"

cd $(dirname ${BASH_SOURCE[0]})/..

docker-compose down --remove-orphans
docker-compose run --rm --service-ports dev $@

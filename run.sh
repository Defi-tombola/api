#!/bin/bash

# Check if 'docker compose' command exists, otherwise use 'docker-compose'
if docker compose version &> /dev/null; then
  COMPOSE="docker compose"
else
  COMPOSE="docker-compose"
fi

compose_file="docker/docker-compose.yml"

# Terminate running services before starting new
$COMPOSE -f $compose_file down grafana-nginx grafana otlp-collector tempo loki prometheus

set -a

case "$1" in
  grafana )
    echo "run.sh: Running Grafana containers"
    # Run services & http
    $COMPOSE -f $compose_file up grafana-nginx grafana otlp-collector tempo loki prometheus --build -d
    ;;
  * )
    echo "run.sh: Invalid argument"
    ;;
esac

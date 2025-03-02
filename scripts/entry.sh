#!/bin/bash

set -e

# Check if $1 is missing
if [ -z "$CONFIG_PATH" ]; then
    echo "Error: CONFIG_PATH environment variable is not set."
    exit 1
fi

case "$1" in
  indexer )
    echo "entry.sh: Config $CONFIG_PATH"
    echo "entry.sh: Running Indexer"
    /app/server --config $CONFIG_PATH indexer
    ;;
  graphql )
    echo "entry.sh: Config $CONFIG_PATH"
    echo "entry.sh: Running GraphQL API"
    /app/server --config $CONFIG_PATH graphql
    ;;
  tasks )
    echo "entry.sh: Config $CONFIG_PATH"
    echo "entry.sh: Running tasrs"
    /app/server --config $CONFIG_PATH tasks
    ;;
  * )
    echo "[ERROR] entry.sh: Invalid argument. Valid options: indexer | graphql | tasks"
    ;;
esac
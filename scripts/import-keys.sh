#!/usr/bin/env bash

set -euo pipefail

# Insert the key using RPC into the keystore
function insert-key-rpc() {
  local node="$1"
  local key_type="$2"
  local rpc_endpoint="$3"

  echo "  - Importing $key_type key for node $node"
  jq --null-input --arg keyType "$key_type" --argfile key "keys/node-$node-$key_type.json" '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "author_insertKey",
    "params": [
      $keyType,
      ($key | .secretPhrase),
      ($key | .publicKey)
    ]
  }' | curl -s -X POST "$rpc_endpoint" -H "Content-Type:application/json;charset=utf-8" -d @-
}

# Insert the key from /path/to/key/file into the keystore
function insert-key-file() {
  local node="$1"
  local key_type="$2"
  local scheme="$([ $key_type == "aura" ] && echo "Sr25519" || echo "Ed25519")"

  echo "  - Importing $key_type key for node $node using scheme $scheme"
  docker-compose -f docker-compose.ibtida.yml run --rm validator-$node \
    key insert \
    --chain=/ibtida.json \
    --key-type=$key_type \
    --scheme=$scheme \
    --suri="$(jq -r '.secretSeed' keys/node-$node-$key_type.json)"
}

function import-node-keys() {
  local node="$1"
  local method="$2"

  if [ "$method" == "file" ]; then
    insert-key-file $node "aura"
    insert-key-file $node "gran"
  elif [ "$method" == "rpc" ]; then
    local rpc_endpoint="http://$(docker-compose -f docker-compose.ibtida.yml port validator-$node 9933)"
    insert-key-rpc $node "aura" "$rpc_endpoint"
    insert-key-rpc $node "gran" "$rpc_endpoint"
  fi
}

METHOD="${1:-file}"

echo "*** Importing Anmol Ibtida chain keys ***"
import-node-keys 1 "$METHOD"
import-node-keys 2 "$METHOD"

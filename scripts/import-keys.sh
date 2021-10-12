#!/usr/bin/env bash

set -euo pipefail

# Insert the key using RPC into the keystore
function insert-key-rpc() {
  local node="$1"
  local key_type="$2"
  local endpoint="$3"

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
  }' | curl -s -X POST "$endpoint" -H "Content-Type:application/json;charset=utf-8" -d @-
}

# Insert the key from /path/to/key/file into the keystore
function insert-key-file() {
  local node="$1"
  local key_type="$2"
  local scheme="$([ $key_type == "aura" ] && echo "Sr25519" || echo "Ed25519")"

  echo "  - Importing $key_type key for node $node using scheme $scheme"
  docker-compose -f docker-compose.ibtida.yml run --rm --entrypoint anmol node-$node \
    key insert \
    --base-path=/var/local/anmol \
    --chain=/var/local/anmol/specs/ibtida.json \
    --key-type=$key_type \
    --scheme=$scheme \
    --suri="$(jq -r '.secretSeed' keys/node-$node-$key_type.json)"
}

function import-node-keys() {
  local node="$1"

  insert-key-file $node "aura"
  insert-key-file $node "gran"

  # local endpoint="http://$(docker-compose -f docker-compose.ibtida.yml port ibtida-$node 9933)"
  # insert-key-rpc $node "aura" "$endpoint"
  # insert-key-rpc $node "gran" "$endpoint"
}

echo "*** Importing Anmol Ibtida chain keys ***"
import-node-keys 1
import-node-keys 2

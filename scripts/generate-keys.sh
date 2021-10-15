#!/usr/bin/env bash

set -euo pipefail

if [[ -x target/release/anmol ]]; then
  anmol="target/release/anmol"
  node_key_path="$(pwd)/keys"
else
  anmol="docker run --rm -v $(pwd)/keys:/var/local/anmol/keys anmolnetwork/anmol-node:ibtida"
  node_key_path="/var/local/anmol/keys"
fi

function generate-keys() {
  local node="$1"

  if [[ -f keys/node-$node.key ]]; then
    echo "  - Found node key for node $node!"
  else
    echo "  - Generating node key for node $node"
    $anmol key generate-node-key --file $node_key_path/node-$node.key
  fi

  if [[ -f keys/node-$node-aura.json ]]; then
    echo "  - Found Aura key for node $node!"
  else
    echo "  - Generating Aura key for node $node"
    $anmol key generate \
      --output-type json \
      --scheme sr25519 > keys/node-$node-aura.json
  fi

  if [[ -f keys/node-$node-gran.json ]]; then
    echo "  - Found Grandpa key for node $node!"
  else
    echo "  - Generating Grandpa key for node $node"
    $anmol key inspect-key \
      --output-type json \
      --scheme ed25519 \
      "$(jq -r .secretPhrase keys/node-$node-aura.json)" > keys/node-$node-gran.json
  fi
}

echo "*** Generating node keys ***"
mkdir -p ./keys
generate-keys 1
generate-keys 2

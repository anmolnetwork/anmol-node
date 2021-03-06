#!/usr/bin/env bash

set -euo pipefail

if [[ -x target/release/anmol ]]; then
  anmol="target/release/anmol"
  chain_spec_path="$(pwd)/chains"
else
  anmol="docker run --rm -v $(pwd)/chains:/var/local/anmol/specs:ro anmolnetwork/anmol-node:ibtida"
  chain_spec_path="/var/local/anmol/specs"
fi

function replace-addresses() {
  jq \
    --argfile node1Aura keys/node-1-aura.json \
    --argfile node2Aura keys/node-2-aura.json \
    --argfile node1Gran keys/node-1-gran.json \
    --argfile node2Gran keys/node-2-gran.json \
    '.genesis.runtime.palletAura.authorities = [($node1Aura | .ss58Address), ($node2Aura | .ss58Address)] |
    .genesis.runtime.palletGrandpa.authorities = [[($node1Gran | .ss58Address), 1], [($node2Gran | .ss58Address), 1]]' | \
  sed 's/5e+27/5000000000000000000000000000/'
}

echo "*** Generating chain spec ***"
echo "  - Replacing Aura and Grandpa keys in spec"
$anmol build-spec \
  --disable-default-bootnode \
  --chain ibtida | \
  replace-addresses > chains/ibtida.json

echo "  - Generating raw chain spec"
$anmol build-spec \
  --disable-default-bootnode \
  --chain $chain_spec_path/ibtida.json \
  --raw > chains/raw/ibtida.json

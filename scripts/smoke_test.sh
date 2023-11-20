#!/bin/bash

SNFORGE_PATH="$1"
SNCAST_PATH="$2"
REPO_URL="$3"
REVISION="$4"

# Check forge

$SNFORGE_PATH init my_project
pushd my_project || exit
scarb remove snforge_std
scarb add snforge_std --git "$REPO_URL" --rev "$REVISION"
$SNFORGE_PATH test || exit
popd || exit

# Check cast

OUTPUT=$($SNCAST_PATH --url http://188.34.188.184:9545/rpc/v0.4 call --contract-address 0x071c8d74edc89330f314f3b1109059d68ebfa68874aa91e9c425a6378ffde00e --function "get_balance")

EXPECTED=$'command: call\nresponse: [0x2]'

if [[ "$OUTPUT" != "$EXPECTED" ]]; then
    exit 1
fi
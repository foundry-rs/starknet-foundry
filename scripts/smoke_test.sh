#!/bin/bash

RPC_URL="$1"
SNFORGE_PATH="$2"
SNCAST_PATH="$3"
REPO_URL="$4"
REVISION="$5"

# Check forge

$SNFORGE_PATH init my_project
pushd my_project || exit
scarb remove snforge_std
scarb add snforge_std --git "$REPO_URL" --rev "$REVISION"
$SNFORGE_PATH test || exit
popd || exit

# Check cast

if ! $SNCAST_PATH --url "$RPC_URL":9545/rpc/v0_6 call \
    --contract-address 0x071c8d74edc89330f314f3b1109059d68ebfa68874aa91e9c425a6378ffde00e \
    --function "get_balance" | grep -q $'command: call\nresponse: [0x2]'; then
  exit 1
fi

#!/bin/bash
set -e

RPC_URL="$1"
SNFORGE_PATH="$2"
SNCAST_PATH="$3"
REPO_URL="$4"
REVISION="$5"
VERSION="$6"

# Check forge github
echo "Check forge github"
$SNFORGE_PATH init my_project_0
pushd my_project_0 || exit
scarb add --dev snforge_std --git "$REPO_URL" --rev "$REVISION"
$SNFORGE_PATH test || exit
popd || exit
scarb cache clean

# Check forge registry prebuild
echo "Check forge registry prebuild"
$SNFORGE_PATH new my_project_1
pushd my_project_1 || exit
sed -i.bak "/snforge_std/ s/\(snforge_std = \).*/\1{ version = \"${VERSION}\", registry = \"https:\/\/scarbs.dev\/\" }/" Scarb.toml
rm Scarb.toml.bak 2> /dev/null
if $SNFORGE_PATH test | grep -q 'Compiling snforge_scarb_plugin'; then
  exit 1
fi 
popd || exit
scarb cache clean

# Check forge registry build
echo "Check forge registry build"

$SNFORGE_PATH new my_project_2
pushd my_project_2 || exit
sed -i.bak "/snforge_std/ s/\(snforge_std = \).*/\1{ version = \"${VERSION}\", registry = \"https:\/\/scarbs.dev\/\" }/" Scarb.toml
sed -i.bak '/^allow-prebuilt-plugins = \["snforge_std"\]$/d' Scarb.toml
rm Scarb.toml.bak 2> /dev/null
$SNFORGE_PATH test || exit
popd || exit
scarb cache clean


# Check cast

if ! $SNCAST_PATH call \
    --url "$RPC_URL" \
    --contract-address 0x06b248bde9ce00d69099304a527640bc9515a08f0b49e5168e2096656f207e1d \
    --function "get" --calldata 0x1 | grep -q $'command: call\nresponse: [0x0]'; then
  exit 1
fi

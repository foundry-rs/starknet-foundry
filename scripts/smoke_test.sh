#!/bin/bash

export DEV_DISABLE_SNFORGE_STD_DEPENDENCY=true

RPC_URL="$1"
SNFORGE_PATH="$2"
SNCAST_PATH="$3"
REPO_URL="$4"
REVISION="$5"
VERSION="$6"

# Check snforge_std from github repository

$SNFORGE_PATH new my_project_0
pushd my_project_0 || exit
scarb add --dev snforge_std --git "$REPO_URL" --rev "$REVISION"
$SNFORGE_PATH test || exit
popd || exit
scarb cache clean

# Check snforge_std from registry with prebuilt plugin

$SNFORGE_PATH new my_project_1
pushd my_project_1 || exit
sed -i.bak "/^\[dev-dependencies\]/a\\
snforge_std = { version = \"=${VERSION}\", registry = \"https://scarbs.dev/\" }\\
" Scarb.toml
rm Scarb.toml.bak 2>/dev/null

test_output=$($SNFORGE_PATH test)
test_exit=$?

echo $test_output

if [[ $test_exit -ne 0 ]] || echo "$test_output" | grep -q 'Compiling snforge_scarb_plugin'; then
    exit 1
fi
popd || exit
scarb cache clean

# Check snforge_std from registry without prebuilt plugin

$SNFORGE_PATH new my_project_2
pushd my_project_2 || exit
sed -i.bak "/^\[dev-dependencies\]/a\\
snforge_std = { version = \"=${VERSION}\", registry = \"https://scarbs.dev/\" }\\
" Scarb.toml
sed -i.bak '/^allow-prebuilt-plugins = \["snforge_std"\]$/d' Scarb.toml
rm Scarb.toml.bak 2>/dev/null
$SNFORGE_PATH test || exit
popd || exit
scarb cache clean

# Check cast

if ! $SNCAST_PATH call \
    --url "$RPC_URL" \
    --contract-address 0x06b248bde9ce00d69099304a527640bc9515a08f0b49e5168e2096656f207e1d \
    --function "get" --calldata 0x1 | grep -q $'Success: Call completed\n\nResponse:     0x0\nResponse Raw: [0x0]'; then
  exit 1
fi

#!/bin/bash
set -e

CAIRO_VERSIONS=["v1", "v2"]
SCARB_VERSIONS=["0.4.1", "0.5.0"]

for cairo_version in $CARIO_VERSIONS; do
  CONTRACTS_DIRECTORY="$(git rev-parse --show-toplevel)/starknet-foundry/crates/cast/tests/data/contracts/$cairo_version"
done


CONTRACTS_DIRECTORY="$(git rev-parse --show-toplevel)/starknet-foundry/crates/cast/tests/data/contracts"
SCARB_VERSION="0.4.1"
ASDF_DATA_DIR=$(asdf info | grep -e "ASDF_DATA_DIR" | awk -F '=' '{print $2}')
SCARB_BIN="$ASDF_DATA_DIR/installs/scarb/$SCARB_VERSION/bin/scarb"

if command -v "$SCARB_BIN" &> /dev/null; then
  for version_dir in "$CONTRACTS_DIRECTORY"/*; do
    for contract_dir in "$version_dir"/*; do
      if ! test -d "$contract_dir"/target; then
        pushd "$contract_dir"
        "$SCARB_BIN" build
        popd
      fi
    done
  done

else
  echo "Please run tests/utils/prepare_for_tests.sh script first!"
fi

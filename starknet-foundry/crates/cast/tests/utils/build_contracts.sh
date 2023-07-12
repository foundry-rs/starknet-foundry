#!/bin/bash
set -e

CAIRO_VERSIONS=("v1" "v2")
SCARB_VERSIONS=("0.4.1" "0.5.2")
ASDF_DATA_DIR=$(asdf info | grep -e "ASDF_DATA_DIR" | awk -F '=' '{print $2}')

for ((i = 0; i < ${#CAIRO_VERSIONS[@]}; i++)); do
  cairo_version="${CAIRO_VERSIONS[i]}"
  scarb_version="${SCARB_VERSIONS[i]}"

  CONTRACTS_DIRECTORY="$(git rev-parse --show-toplevel)/starknet-foundry/crates/cast/tests/data/contracts/$cairo_version"
  SCARB_BIN="$ASDF_DATA_DIR/installs/scarb/$scarb_version/bin/scarb"

  if command -v "$SCARB_BIN" &> /dev/null; then
    for contract_dir in "$CONTRACTS_DIRECTORY"/*; do
      if ! test -d "$contract_dir"/target; then
        pushd "$contract_dir"
        "$SCARB_BIN" build
        popd
      fi
    done

  else
    echo "Please run tests/utils/prepare_for_tests.sh script first!"
  fi

done

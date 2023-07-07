#!/bin/bash
set -e

CONTRACTS_DIRECTORY="$(git rev-parse --show-toplevel)/starknet-foundry/crates/cast/tests/data/contracts"

if command -v scarb &> /dev/null; then
  for contract_dir in "$CONTRACTS_DIRECTORY"/*; do
    if ! test -d "$contract_dir"/target; then
      pushd "$contract_dir"
      scarb build
      popd
    fi
  done

else
  echo "Please run tests/utils/prepare_for_tests.sh script first!"
fi

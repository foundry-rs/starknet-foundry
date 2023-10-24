#!/bin/bash
set -e

SCARB_VERSION="0.5.2"
CONTRACTS_DIRECTORY="$(git rev-parse --show-toplevel)/crates/cast/tests/data/contracts/"

if ! scarb --version | grep -qF "$SCARB_VERSION"; then
  echo "Please run tests/utils/prepare_for_tests.sh script first!"
  echo "wrong version of scarb found, required version is $SCARB_VERSION"
  exit 1
fi

for contract_dir in "$CONTRACTS_DIRECTORY"/*; do
  if ! test -d "$contract_dir"/target && [[ "$contract_dir" != *"fails"* ]]; then
    pushd "$contract_dir"
    scarb build
    popd
  fi
done

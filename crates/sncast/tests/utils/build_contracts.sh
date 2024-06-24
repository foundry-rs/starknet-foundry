#!/bin/bash
set -e

SCARB_VERSION="2.5.4"
REPO_ROOT=$(git rev-parse --show-toplevel)
CONTRACTS_DIRECTORY="${REPO_ROOT}/crates/sncast/tests/data/contracts/"

if ! scarb --version | grep -qF "$SCARB_VERSION"; then
  echo "Wrong version of scarb found, required version is $SCARB_VERSION"
  exit 1
fi

if [ ! -d "${REPO_ROOT}/crates/sncast/tests/utils/devnet/bin" ]; then
  echo "Devnet not found - please run ${REPO_ROOT}/scripts/install_devnet.sh script first!"
  exit 1
fi

for contract_dir in "$CONTRACTS_DIRECTORY"/*; do
  if ! test -d "$contract_dir"/target && [[ "$contract_dir" != *"fails"* ]]; then
    pushd "$contract_dir"
    scarb build
    popd
  fi
done

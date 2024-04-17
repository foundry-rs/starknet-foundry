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

function run_usc_inside() {
  target_dir=$1

  for contract_filename in `ls  $target_dir/*.contract_class.json`; do
    contract_name=`basename $contract_filename .contract_class.json`
    universal-sierra-compiler compile-contract --sierra-path "$contract_filename" --output-path "$target_dir/$contract_name.compiled_contract_class.json"
  done
}

for contract_dir in "$CONTRACTS_DIRECTORY"/*; do
  if ! test -d "$contract_dir"/target && [[ "$contract_dir" != *"fails"* ]]; then
    pushd "$contract_dir"
    scarb build

    run_usc_inside "$contract_dir/target/dev"

    popd
  fi
done

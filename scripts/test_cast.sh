#!/bin/sh

set -e

cd -P -- "$(dirname -- "$0")"

./prepare_for_tests.sh
asdf install scarb 0.5.0-alpha.0
asdf global scarb 0.5.0-alpha.0
cd ../starknet-foundry/crates/cast && cargo test

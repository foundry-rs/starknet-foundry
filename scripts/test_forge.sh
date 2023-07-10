#!/bin/sh

set -e

cd -P -- "$(dirname -- "$0")"

asdf install scarb 0.5.1
asdf global scarb 0.5.1
cd ../starknet-foundry/crates/forge && cargo test
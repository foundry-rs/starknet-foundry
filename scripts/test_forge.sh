#!/bin/sh

set -e

cd -P -- "$(dirname -- "$0")"

asdf install scarb 0.7.0
asdf global scarb 0.7.0
cargo test --release -p forge

#!/bin/sh

set -e

cd -P -- "$(dirname -- "$0")"

asdf install scarb 0.6.0
asdf global scarb 0.6.0
cd ../crates/forge && cargo test

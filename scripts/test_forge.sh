#!/bin/sh

set -e

cd -P -- "$(dirname -- "$0")"

asdf install scarb 0.5.2
asdf global scarb 0.5.2
cd ../crates/forge && cargo test

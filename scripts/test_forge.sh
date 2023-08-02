#!/bin/sh

set -e

cd -P -- "$(dirname -- "$0")"

asdf install scarb 0.6.0-alpha.2
asdf global scarb 0.6.0-alpha.2
cd ../crates/forge && cargo test

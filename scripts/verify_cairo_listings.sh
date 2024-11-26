#!/bin/bash
set -e

# TODO: ATM there is a bug in Scarb - .so library isn't getting build, so we need to build it manually
# Should be removed once the bug is fixed
cargo build --release --manifest-path ./crates/snforge-scarb-plugin/Cargo.toml

for d in ./docs/listings/*; do (cd "$d" && scarb test); done

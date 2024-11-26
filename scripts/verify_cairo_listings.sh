#!/bin/bash
set -xe

# TODO(#2718)
cargo build --release --manifest-path ./crates/snforge-scarb-plugin/Cargo.toml

for d in ./docs/listings/*; do (cd "$d" && scarb check); done

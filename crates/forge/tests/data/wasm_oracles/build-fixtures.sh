#!/usr/bin/env sh
set -ex

# Run this script to generate wasm fixtures from their sources.
# Prebuilt fixtures are expected to be commited to the repository.

cd "$(dirname "$0")"

cargo build --manifest-path=wasm_oracle/Cargo.toml --release --target wasm32-wasip2
cp wasm_oracle/target/wasm32-wasip2/release/wasm_oracle.wasm .

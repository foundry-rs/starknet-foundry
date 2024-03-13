#!/bin/bash
set -e

DEVNET_INSTALL_DIR="$(git rev-parse --show-toplevel)/crates/sncast/tests/utils/devnet"
DEVNET_REPO="https://github.com/0xSpaceShard/starknet-devnet-rs.git"
DEVNET_REV="37dc6e6"

# https://github.com/0xSpaceShard/starknet-devnet-rs/blob/main/.cargo/config.toml
export STARKNET_VERSION="0.13.0"
export RPC_SPEC_VERSION="0.7.0-rc2"

cargo install --locked --git "$DEVNET_REPO" --rev "$DEVNET_REV" --root "$DEVNET_INSTALL_DIR"

echo "All done!"
exit 0

#!/bin/bash
set -e

DEVNET_INSTALL_DIR="$(git rev-parse --show-toplevel)/crates/sncast/tests/utils/devnet"
DEVNET_REPO="https://github.com/0xSpaceShard/starknet-devnet-rs.git"
DEVNET_REV="64c425b"

# https://github.com/0xSpaceShard/starknet-devnet-rs/blob/main/.cargo/config.toml
export STARKNET_VERSION="0.12.3"
export RPC_SPEC_VERSION="0.5.1"

cargo install --locked --git "$DEVNET_REPO" --rev "$DEVNET_REV" --root "$DEVNET_INSTALL_DIR"

echo "All done!"
exit 0

#!/bin/bash
set -e

DEVNET_INSTALL_DIR="$(git rev-parse --show-toplevel)/crates/sncast/tests/utils/devnet"
DEVNET_REPO="https://github.com/0xSpaceShard/starknet-devnet-rs.git"
DEVNET_REV="37dc6e6"

cargo install --locked --git "$DEVNET_REPO" --rev "$DEVNET_REV" --root "$DEVNET_INSTALL_DIR" --force

echo "All done!"
exit 0

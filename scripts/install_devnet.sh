#!/bin/bash
set -e

DEVNET_INSTALL_DIR="$(git rev-parse --show-toplevel)/crates/cast/tests/utils/devnet"
DEVNET_REPO="https://github.com/0xSpaceShard/starknet-devnet-rs.git"
DEVNET_REV="72c11f6"

cargo install --git "$DEVNET_REPO" --rev "$DEVNET_REV" --root "$DEVNET_INSTALL_DIR"

echo "All done!"
exit 0

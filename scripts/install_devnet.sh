#!/bin/bash
set -e

DEVNET_INSTALL_DIR="$(git rev-parse --show-toplevel)/crates/cast/tests/utils/devnet"
DEVNET_REPO="https://github.com/0xSpaceShard/starknet-devnet-rs.git"
DEVNET_REV="72c11f6"

git clone "$DEVNET_REPO" "$DEVNET_INSTALL_DIR/$DEVNET_REV" || echo "Repo already checked out!"
pushd "$DEVNET_INSTALL_DIR/$DEVNET_REV"
cargo build --release
popd

mkdir -p "$DEVNET_INSTALL_DIR/bin/"
cp "$DEVNET_INSTALL_DIR/$DEVNET_REV/target/release/starknet-devnet" "$DEVNET_INSTALL_DIR/bin/."

echo "All done!"
exit 0

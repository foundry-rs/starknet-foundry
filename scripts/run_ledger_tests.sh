#!/bin/sh

# Runs Ledger tests inside the `ledger-app-dev-tools` Docker image, which provides
# Speculos (a Ledger device emulator) and all required tooling. No physical Ledger
# device is needed.
#
# Prerequisites:
#   - Docker installed and running
#   - A pre-built Nano X ELF binary at crates/sncast/tests/data/ledger-app/nanox.elf
#     (see docs/src/development/environment-setup.md for how to build it)
#
# The first run compiles the workspace and installs the Scarb toolchain via starkup,
# so it may take a while. Later runs reuse the Docker cache volumes.

set -eu

docker run --rm -it \
    -v "$(pwd):/workspace" \
    -v ledger_build_cache:/workspace/target \
    -v "$HOME/.cargo/registry:/root/.cargo/registry" \
    -v ledger_asdf_cache:/root/.asdf \
    -v ledger_local_cache:/root/.local \
    -w /workspace \
    -e CARGO_TARGET_DIR=/workspace/target \
    ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:5.3.16 \
    bash -c '
        [ -f /opt/.cargo/env ] && source /opt/.cargo/env
        if ! [ -x "$HOME/.asdf/shims/scarb" ]; then
            curl --proto "=https" --tlsv1.2 -sSf https://sh.starkup.sh | sh -s -- --yes
        fi
        export PATH="$HOME/.asdf/shims:$HOME/.asdf/bin:$HOME/.local/bin:$PATH"
        [ -f "$HOME/.asdf/asdf.sh" ] && source "$HOME/.asdf/asdf.sh"
        export RUSTUP_TOOLCHAIN=stable
        SCARB_VERSION=$(grep "scarb " /workspace/.tool-versions | cut -d " " -f 2)
        DEVNET_VERSION=$(grep "starknet-devnet " /workspace/.tool-versions | cut -d " " -f 2)
        asdf set -u scarb "$SCARB_VERSION"
        asdf set -u starknet-devnet "$DEVNET_VERSION"
        cargo test -p sncast --features ledger-emulator --test main ledger -- --nocapture --ignored
    '

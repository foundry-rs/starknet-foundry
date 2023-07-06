#!/bin/bash
set -e

COMPILER_DIRECTORY="$(git rev-parse --show-toplevel)/cast/tests/utils/"
CAIRO_REPO="https://github.com/starkware-libs/cairo/releases/download"
COMPILER_VERSION="v1.1.1"
SCARB_VERSION="0.4.1"

if ! which starknet-devnet > /dev/null 2>&1; then
  echo "starknet-devnet not found, exiting."
  exit 1
fi

if command -v asdf &> /dev/null; then
  asdf plugin add scarb https://github.com/software-mansion/asdf-scarb.git
  asdf install scarb "$SCARB_VERSION"
  asdf global scarb "$SCARB_VERSION"
  scarb --version
else
  printf "Please install asdf\n https://asdf-vm.com/guide/getting-started.html#_2-download-asdf\n"
  exit 1
fi

if [ ! -x "$COMPILER_DIRECTORY/cairo/bin/starknet-sierra-compile" ]; then
  if [[ $(uname -s) == 'Darwin' ]]; then
    wget "$CAIRO_REPO/$COMPILER_VERSION/release-aarch64-apple-darwin.tar" -P "$COMPILER_DIRECTORY"
    pushd "$COMPILER_DIRECTORY"
    tar -xvf "$COMPILER_DIRECTORY/release-aarch64-apple-darwin.tar" cairo/bin/starknet-sierra-compile
    popd

  elif [[ $(uname -s) == 'Linux' ]]; then
    wget "$CAIRO_REPO/$COMPILER_VERSION/release-x86_64-unknown-linux-musl.tar.gz" -P "$COMPILER_DIRECTORY"
    pushd "$COMPILER_DIRECTORY"
    tar -xzvf "$COMPILER_DIRECTORY/release-x86_64-unknown-linux-musl.tar.gz" cairo/bin/starknet-sierra-compile
    popd
  fi
fi

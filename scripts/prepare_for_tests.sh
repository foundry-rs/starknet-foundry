#!/bin/bash
set -e

COMPILER_DIRECTORY="$(git rev-parse --show-toplevel)/crates/cast/tests/utils/compiler"
CAIRO_REPO="https://github.com/starkware-libs/cairo/releases/download"

SCARB_VERSION="0.5.2"
DEVNET_VERSION="0.5.5"

if ! which starknet-devnet > /dev/null 2>&1; then
  echo "starknet-devnet not found, please install with version $DEVNET_VERSION"
  echo "https://0xspaceshard.github.io/starknet-devnet/docs/intro"
  exit 1
fi

if ! grep -q "$(starknet-devnet --version)" <<< "$DEVNET_VERSION"; then
  echo "wrong version of starknet-devnet found, required version is $DEVNET_VERSION"
  exit 1
fi

if ! which scarb > /dev/null 2>&1; then
  echo "scarb not found, please install with version $SCARB_VERSION"
  echo "https://docs.swmansion.com/scarb/download.html"
  exit 1
fi

if ! scarb --version | grep -qF "$SCARB_VERSION"; then
  echo "wrong version of scarb found, required version is $SCARB_VERSION"
  exit 1
fi


function get_compiler () {
  compiler_version=$1
  filename=$2

  wget "$CAIRO_REPO/$compiler_version/$filename" -P "$COMPILER_DIRECTORY/scarb-$SCARB_VERSION"
  pushd "$COMPILER_DIRECTORY/scarb-$SCARB_VERSION"
  tar -xvf "$COMPILER_DIRECTORY/scarb-$SCARB_VERSION/$filename" cairo/bin/starknet-sierra-compile
  popd
}

compiler_version="v$(scarb --version | grep -e "cairo:" | awk '{print $2}')"

if [ ! -x "$COMPILER_DIRECTORY/scarb-$SCARB_VERSION/cairo/bin/starknet-sierra-compile" ]; then
  if [[ $(uname -s) == 'Darwin' ]]; then
    get_compiler "$compiler_version" "release-aarch64-apple-darwin.tar"

  elif [[ $(uname -s) == 'Linux' ]]; then
    get_compiler "$compiler_version" "release-x86_64-unknown-linux-musl.tar.gz"

  else
    echo "System $(uname -s) currently not supported"
    exit 1
  fi
fi

echo "All done!"
exit 0


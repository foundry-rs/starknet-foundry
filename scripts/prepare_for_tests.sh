#!/bin/bash
set -e

COMPILER_DIRECTORY="$(git rev-parse --show-toplevel)/starknet-foundry/crates/cast/tests/utils/compiler"
CAIRO_REPO="https://github.com/starkware-libs/cairo/releases/download"

COMPILER_VERSIONS=("v1.1.1" "v2.0.2")
SCARB_VERSIONS=("0.4.1" "0.5.2")
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

if ! command -v asdf 2>&1 /dev/null; then
  echo "asdf not found, please install"
  echo "https://asdf-vm.com/guide/getting-started.html#_2-download-asdf"
  exit 1
fi

if command -v scarb 2>&1 /dev/null; then
  installed_versions=$(asdf list scarb)

  for scarb_version in "${SCARB_VERSIONS[@]}"; do
    if ! grep -q "$scarb_version" <<< "$installed_versions"; then
      asdf install scarb "$scarb_version"
    fi
  done

else
  asdf plugin add scarb https://github.com/software-mansion/asdf-scarb.git
  for scarb_version in "${SCARB_VERSIONS[@]}"; do
    asdf install scarb "$scarb_version"
  done
fi

for compiler_version in "${COMPILER_VERSIONS[@]}"; do
  if [ ! -x "$COMPILER_DIRECTORY/cairo/$compiler_version/bin/starknet-sierra-compile" ]; then
    if [[ $(uname -s) == 'Darwin' ]]; then
      wget "$CAIRO_REPO/$compiler_version/release-aarch64-apple-darwin.tar" -P "$COMPILER_DIRECTORY/$compiler_version"
      pushd "$COMPILER_DIRECTORY/$compiler_version"
      tar -xvf "$COMPILER_DIRECTORY/$compiler_version/release-aarch64-apple-darwin.tar" cairo/bin/starknet-sierra-compile
      popd

    elif [[ $(uname -s) == 'Linux' ]]; then
      wget "$CAIRO_REPO/$compiler_version/release-x86_64-unknown-linux-musl.tar.gz" -P "$COMPILER_DIRECTORY/$compiler_version"
      pushd "$COMPILER_DIRECTORY/$compiler_version"
      tar -xzvf "$COMPILER_DIRECTORY/$compiler_version/release-x86_64-unknown-linux-musl.tar.gz" cairo/bin/starknet-sierra-compile
      popd

    else
      echo "System $(uname -s) currently not supported"
      exit 1
    fi
  fi
done

asdf global scarb 0.4.1

echo "All done!"
exit 0


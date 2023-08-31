#!/bin/bash
set -e

COMPILER_DIRECTORY="$(git rev-parse --show-toplevel)/crates/cast/tests/utils/compiler"
CAIRO_REPO="https://github.com/starkware-libs/cairo/releases/download"

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

function get_compiler () {
  compiler_version=$1
  filename=$2

  wget "$CAIRO_REPO/$compiler_version/$filename" -P "$COMPILER_DIRECTORY/$compiler_version"
  pushd "$COMPILER_DIRECTORY/$compiler_version"
  tar -xvf "$COMPILER_DIRECTORY/$compiler_version/$filename" cairo/bin/starknet-sierra-compile
  popd
}

for scarb_version in "${SCARB_VERSIONS[@]}"; do

  asdf local scarb "$scarb_version"
  compiler_version="v$(scarb --version | grep -e "cairo:" | awk '{print $2}')"

  if [ ! -x "$COMPILER_DIRECTORY/$compiler_version/cairo/bin/starknet-sierra-compile" ]; then
    if [[ $(uname -s) == 'Darwin' ]]; then
      get_compiler "$compiler_version" "release-aarch64-apple-darwin.tar"

    elif [[ $(uname -s) == 'Linux' ]]; then
      get_compiler "$compiler_version" "release-x86_64-unknown-linux-musl.tar.gz"

    else
      echo "System $(uname -s) currently not supported"
      exit 1
    fi
  fi
done

asdf global scarb 0.4.1
rm .tool-versions

echo "All done!"
exit 0


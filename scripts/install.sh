#!/bin/sh
# shellcheck shell=dash
# shellcheck disable=SC2039

# This is just a little script that can be downloaded from the internet to install Starknet-Foundry.
# It just does platform detection, downloads the release archive, extracts it and tries to make
# the `forge` and `cast` binaries available in $PATH in least invasive way possible.
#
# It runs on Unix shells like {a,ba,da,k,z}sh. It uses the common `local` extension.
# Note: Most shells limit `local` to 1 var per line, contra bash.
#
# Most of this code is based on/copy-pasted from rustup and protostar installers.


cd -P -- "$(dirname -- "$0")"  # change working directory to `scripts`
./forge_install.sh
./cast_install.sh

#!/usr/bin/env bash

SNFOUNDRY_ROOT="$(git rev-parse --show-toplevel)"

pushd "$SNFOUNDRY_ROOT" || exit

find . -type f -name "Scarb.toml" -execdir sh -c '
  echo "Running \"scarb fmt\" in directory: $PWD"
  scarb fmt
' \;

popd || exit

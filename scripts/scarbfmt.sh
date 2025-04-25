#!/usr/bin/env bash

output=$(find . -type f -name "Scarb.toml" -execdir sh -c '
    echo "Running \"scarb fmt\" in directory: $PWD"
    scarb fmt
' \;)
echo "$output"
if grep -iq "Diff" <<<"$output"; then
  exit 1
fi

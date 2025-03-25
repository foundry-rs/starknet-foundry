#!/usr/bin/env bash

SNFOUNDRY_ROOT="$(git rev-parse --show-toplevel)"

pushd "$SNFOUNDRY_ROOT" || exit

output=$(find . -type f -name "Scarb.toml" -execdir sh -c '
    echo "Running \"scarb fmt --check\" in directory: $PWD"
    scarb fmt --check
' \;)

echo "$output"

if grep -iq "Diff" <<< "$output"; then
    exit 1
fi
exit 0
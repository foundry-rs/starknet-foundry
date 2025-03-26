#!/usr/bin/env bash

SNFOUNDRY_ROOT="$(git rev-parse --show-toplevel)"

pushd "$SNFOUNDRY_ROOT" || exit

SKIPPED_PACKAGES=(
  "backtrace_panic"
  "build_fails"
  "missing_field"
  "trace_resources"
  "coverage_project"
  "snforge-scarb-plugin"
  "scripts"
)

export SKIPPED_PACKAGES_STR="${SKIPPED_PACKAGES[*]}"

find . -type f -name "Scarb.toml" -execdir bash -c '
  current_package=$(basename "$PWD")
  IFS=" " read -r -a skipped <<< "$SKIPPED_PACKAGES_STR"
  for pkg in "${skipped[@]}"; do
    if [[ "$current_package" == "$pkg" ]]; then
      echo "Skipping package: $current_package"
      exit 0
    fi
  done
  echo "Running \"scarb lint\" in directory: $PWD"
  scarb lint
' \;

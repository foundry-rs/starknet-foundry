#!/usr/bin/env bash

SNFOUNDRY_ROOT="$(git rev-parse --show-toplevel)"

pushd "$SNFOUNDRY_ROOT" || exit

SKIPPED_PACKAGES=(
  "backtrace_panic" # This package has code which results in compiler (not lint) warnings
  "build_fails" # This package fails to compile on purpose
  "missing_field" # This package fails to compile on purpose
  "trace_resources" # This package has code which results in compiler (not lint) warnings
  "coverage_project" # This package has code which results in compiler (not lint) warnings
  "snforge-scarb-plugin"
)

export SKIPPED_PACKAGES_STR="${SKIPPED_PACKAGES[*]}"

output=$(find . -type f -name "Scarb.toml" -execdir bash -c '
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
' \;)

echo "$output"

# Check for lint warnings, we need to do it this way because `scarb lint` returns 0 even if there are warnings
if grep -iq "warning: Plugin diagnostic:" <<< "$output"; then
    exit 1
# Check for missing `edition`` field
elif grep -iq "warn: `edition` field not set in `[package]` section for package" <<< "$output"; then
    exit 1
elif grep -iq "error: no such command: \`lint\`" <<< "$output"; then
    exit 1
fi

#!/bin/bash
set -xe

# TODO(#3469): Some packages are now skipped because they use newer Scarb.
# Once Cairo 2.12 is the minimal version, we should remove this logic.
skipped_packages=("deployment_with_constructor_args")

is_skipped() {
  local dir=$1
  for skip in "${skipped_packages[@]}"; do
    if [[ $(basename "$dir") == "$skip" ]]; then
      return 0
    fi
  done
  return 1
}

# TODO(#2718)
for d in ./docs/listings/*; do
  if is_skipped "$d"; then
    echo "Skipping build in: $d"
  else
    (cd "$d" && scarb build)
  fi
done
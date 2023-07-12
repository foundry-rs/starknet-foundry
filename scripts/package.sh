#!/usr/bin/env bash
set -euxo pipefail

TARGET="$1"
STAGING="$2"

rm -rf "$STAGING"
mkdir -p "$STAGING/bin"

bin_ext=""
[[ "$TARGET" == *-windows-* ]] && bin_ext=".exe"

binary_crates=("forge" "cast")
for crate in "${binary_crates[@]}"; do
  cp "./starknet-foundry/target/${TARGET}/release/${crate}${bin_ext}" "$STAGING/bin/"
done

cp -r README.md "$STAGING/"

if [[ "$TARGET" == *-windows-* ]]; then
  7z a "${STAGING}.zip" "$STAGING"
else
  tar czvf "${STAGING}.tar.gz" "$STAGING"
fi

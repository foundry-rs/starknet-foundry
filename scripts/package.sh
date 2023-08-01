#!/usr/bin/env bash
set -euxo pipefail

TARGET="$1"
PKG_FULL_NAME="$2"

rm -rf "$PKG_FULL_NAME"
mkdir -p "$PKG_FULL_NAME/bin"

bin_ext=""
[[ "$TARGET" == *-windows-* ]] && bin_ext=".exe"

binary_crates=("snforge" "sncast")
for crate in "${binary_crates[@]}"; do
  cp "./target/${TARGET}/release/${crate}${bin_ext}" "$PKG_FULL_NAME/bin/"
done

cp -r README.md "$PKG_FULL_NAME/"

if [[ "$TARGET" == *-windows-* ]]; then
  7z a "${PKG_FULL_NAME}.zip" "$PKG_FULL_NAME"
else
  tar czvf "${PKG_FULL_NAME}.tar.gz" "$PKG_FULL_NAME"
fi

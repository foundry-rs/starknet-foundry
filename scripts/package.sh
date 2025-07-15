#!/usr/bin/env bash
set -euxo pipefail

TARGET="$1"
PKG_FULL_NAME="$2"

rm -rf "$PKG_FULL_NAME"
mkdir -p "$PKG_FULL_NAME/bin"

binary_crates=("snforge" "sncast")
for crate in "${binary_crates[@]}"; do
  cp "./target/${TARGET}/release/${crate}" "$PKG_FULL_NAME/bin/"
done

cp -r README.md LICENSE "$PKG_FULL_NAME/"

tar czvf "${PKG_FULL_NAME}.tar.gz" "$PKG_FULL_NAME"

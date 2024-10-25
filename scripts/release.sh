#!/bin/bash

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Error: Version argument is required"
    echo "Usage: $0 <version>"
    exit 1
fi

sed -i.bak "s/## \[Unreleased\]/## \[Unreleased\]\n\n## \[${VERSION}\] - $(TZ=Europe/Krakow date '+%Y-%m-%d')/" CHANGELOG.md
rm CHANGELOG.md.bak 2> /dev/null

sed -i.bak "/\[workspace.package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" Cargo.toml
rm Cargo.toml.bak 2> /dev/null

declare -a scarb_files=(
    "sncast_std/Scarb.toml"
    "snforge_std/Scarb.toml"
    "crates/snforge-scarb-plugin/Scarb.toml"
)

for file in "${scarb_files[@]}"; do
    sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" "$file"
    rm "${file}.bak" 2> /dev/null
done

scarb --manifest-path sncast_std/Scarb.toml build
scarb --manifest-path snforge_std/Scarb.toml build

cargo update -p forge
cargo update -p sncast

declare -a doc_files=(
    "./docs/src/getting-started/first-steps.md"
    "./docs/src/starknet/script.md"
    "./docs/src/appendix/scarb-toml.md"
    "./docs/src/appendix/cheatcodes.md"
    "./docs/src/appendix/snforge-library.md"
    "./docs/src/testing/contracts.md"
    "./docs/src/testing/using-cheatcodes.md"
)

echo "Updating version in documentation files..."
for file in "${doc_files[@]}"; do
    if [ -f "$file" ]; then
        sed -i.bak -E 's/(version[[:space:]]*=[[:space:]]*"|version:[[:space:]]*)[0-9]+\.[0-9]+\.[0-9]+/\1'"${VERSION}"'/' "$file"
        rm "${file}.bak" 2> /dev/null
        echo "Updated $file"
    else
        echo "Warning: $file not found"
    fi
done

echo "Version update complete!"
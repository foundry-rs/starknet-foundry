#!/bin/bash

VERSION=$1

sed -i.bak "s/## \[Unreleased\]/## \[Unreleased\]\n\n## \[${VERSION}\] - $(TZ=Europe/Krakow date '+%Y-%m-%d')/" CHANGELOG.md
rm CHANGELOG.md.bak 2> /dev/null

sed -i.bak "/\[workspace.package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" Cargo.toml
rm Cargo.toml.bak 2> /dev/null

sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" sncast_std/Scarb.toml
rm sncast_std/Scarb.toml.bak 2> /dev/null

scarb --manifest-path sncast_std/Scarb.toml build

sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" snforge_std/Scarb.toml
rm snforge_std/Scarb.toml.bak 2> /dev/null

sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" crates/snforge-scarb-plugin/Scarb.toml
rm crates/snforge-scarb-plugin/Scarb.toml.bak 2> /dev/null

sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" crates/snforge-scarb-plugin/Cargo.toml
rm crates/snforge-scarb-plugin/Cargo.toml.bak 2> /dev/null

sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" snforge_std_deprecated/Scarb.toml
rm snforge_std_deprecated/Scarb.toml.bak 2> /dev/null

sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" crates/snforge-scarb-plugin-deprecated/Scarb.toml
rm crates/snforge-scarb-plugin-deprecated/Scarb.toml.bak 2> /dev/null

sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" crates/snforge-scarb-plugin-deprecated/Cargo.toml
rm crates/snforge-scarb-plugin-deprecated/Cargo.toml.bak 2> /dev/null

sed -i.bak "/\[preprocessor.variables.variables\]/,/snforge_std_version =/ s/snforge_std_version = \".*/snforge_std_version = \"${VERSION}\"/" docs/book.toml
rm docs/book.toml.bak 2> /dev/null

# start: Update cache test data
VERSION_UNDERSCORED=$(echo "$VERSION" | tr '.' '_')

DIRECTORY="crates/forge/tests/data/forking/.snfoundry_cache"
OLD_FILE_PATH=$(find "$DIRECTORY" -type f -regex '.*_v[0-9][0-9_]*\.json')
NEW_FILE_PATH=$(echo "$OLD_FILE_PATH" | sed -E "s/_v[0-9_]+\.json$/_v${VERSION_UNDERSCORED}.json/")

mv "$OLD_FILE_PATH" "$NEW_FILE_PATH"

sed -i.bak -E "s/\"cache_version\":\"[0-9_]+\"/\"cache_version\":\"${VERSION_UNDERSCORED}\"/" "$NEW_FILE_PATH"
rm "$NEW_FILE_PATH.bak" 2> /dev/null
# end

scarb --manifest-path snforge_std/Scarb.toml build
scarb --manifest-path snforge_std_deprecated/Scarb.toml build --ignore-cairo-version

cargo update
cargo update --manifest-path crates/snforge-scarb-plugin/Cargo.toml
cargo update --manifest-path crates/snforge-scarb-plugin-deprecated/Cargo.toml

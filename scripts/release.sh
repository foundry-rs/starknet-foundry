#!/bin/bash

VERSION=$1

parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
echo "Was starknet.rs package updated during this release cycle?"
select yn in "Yes" "No"; do
    case $yn in
        Yes ) echo "What is the spec version it supports?";read spec_version;echo $spec_version > "$parent_path/../crates/forge/expected-rpc-version";break;;
        No ) break;;
    esac
done

sed -i.bak "s/## \[Unreleased\]/## \[Unreleased\]\n\n## \[${VERSION}\] - $(TZ=Europe/Krakow date '+%Y-%m-%d')/" CHANGELOG.md
rm CHANGELOG.md.bak 2> /dev/null

sed -i.bak "/\[workspace.package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" Cargo.toml
rm Cargo.toml.bak 2> /dev/null

sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" sncast_std/Scarb.toml
rm sncast_std/Scarb.toml.bak 2> /dev/null

scarb --manifest-path sncast_std/Scarb.toml build

sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" snforge_std/Scarb.toml
rm snforge_std/Scarb.toml.bak 2> /dev/null

scarb --manifest-path snforge_std/Scarb.toml build

cargo update -p forge
cargo update -p sncast

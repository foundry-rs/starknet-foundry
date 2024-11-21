#!/bin/bash

PLUGIN_FILE_PATH="../crates/snforge-scarb-plugin/Scarb.toml"
SNFORGE_STD_PATH="../snforge_std/Scarb.toml"

VERSION=$(grep version "$PLUGIN_FILE_PATH" | cut -d '"' -f 2)

sed -i.bak "/snforge_scarb_plugin/ s/\(snforge_scarb_plugin = \).*/\1\"^${VERSION}\"/" $SNFORGE_STD_PATH

rm ${SNFORGE_STD_PATH}.bak 2> /dev/null

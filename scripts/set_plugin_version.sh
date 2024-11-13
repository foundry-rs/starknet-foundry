#!/bin/bash

PLUGIN_FILE_PATH="../crates/snforge-scarb-plugin/Scarb.toml"

VERSION=$(cat "$PLUGIN_FILE_PATH" | grep version | cut -d '"' -f 2)

STD_FILE_PATH="../snforge_std/Scarb.toml"

sed -i.bak "/snforge_scarb_plugin/ s/\(snforge_scarb_plugin = \).*/\1\"^${VERSION}\"/" $STD_FILE_PATH

rm ${STD_FILE_PATH}.bak 2> /dev/null

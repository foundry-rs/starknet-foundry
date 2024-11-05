#!/bin/bash

SOURCE=$1

MANIFEST_FILE="../snforge_std/Scarb.toml"

sed -i.bak "/snforge_scarb_plugin/ s/\(snforge_scarb_plugin = \).*/\1\"${SOURCE}\"/" $MANIFEST_FILE

rm ${MANIFEST_FILE}.bak 2> /dev/null

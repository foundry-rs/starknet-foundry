#!/bin/bash

# TODO(#2781)

pushd docs

OUTPUT=$(mdbook build 2>&1)

echo "$OUTPUT"

# Check for errors in both mdbook 0.4 format ([ERROR]) and mdbook 0.5 format (ERROR)
if echo "$OUTPUT" | grep -qE "\[ERROR\]|^ERROR "; then
    exit 1
fi

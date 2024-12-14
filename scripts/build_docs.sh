#!/bin/bash

pushd docs

OUTPUT=$(mdbook build 2>&1)

echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "\[ERROR\]"; then
    exit 1
fi

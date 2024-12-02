#!/bin/bash
set -xe

# TODO(#2718)
for d in ./docs/listings/*; do
    if [ -f "$d/Scarb.toml" ]; then
        (cd "$d" && scarb build);
    fi    
done

#!/bin/bash
set -xe

# TODO(#2718)
for d in ./docs/listings/*; do (cd "$d" && scarb check); done

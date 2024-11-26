#!/bin/bash
set -xe

for d in ./docs/listings/*; do (cd "$d" && scarb test); done

#!/bin/bash
set -e

for d in ./docs/listings/*; do (cd "$d" && scarb test); done

#!/bin/bash
set -xe

# TODO(#3469): Remove this whole script once Cairo 2.12 is the minimal version.
scarb_2_12_packages=("deployment_with_constructor_args")

for package in "${scarb_2_12_packages[@]}"; do (cd "$d" && scarb build); done

#!/bin/bash
set -xe

scarb_2_12_packages=("deployment_with_constructor_args")

# iterate overy the packages from scarb_2_12_packages
for package in "${scarb_2_12_packages[@]}"; do (cd "$d" && scarb build); done
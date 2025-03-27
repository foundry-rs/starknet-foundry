#!/usr/bin/env bash

SNFOUNDRY_ROOT="$(git rev-parse --show-toplevel)"

pushd "$SNFOUNDRY_ROOT" || exit

output=$(target/release/snforge new --template 2>&1)

echo "$output"

templates=$(echo "$output" | grep "possible values:" | sed -E 's/.*possible values: (.*)]/\1/')
IFS=', ' read -r -a templates <<< "$templates"

for template in "${templates[@]}"; do
    echo "Initializing package for template: $template"
    package_name="${template//-/_}_test" # Replace hyphens with underscores, we can't have hyphens in package directory name
    target/release/snforge new --template $template $package_name

    pushd $package_name

    echo "Running \"scarb check\" in directory: $PWD"
    scarb check

    popd
done

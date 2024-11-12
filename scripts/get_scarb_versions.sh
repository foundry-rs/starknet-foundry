#!/bin/bash
set -e

# This script is used to find the following Scarb versions:
# The current major.minor version along with all its patch versions
# The latest patch versions for the two versions preceding the current one

function get_all_patch_versions() {
  # Omit version 2.8.0 as it has a bug when using the `assert_macros` package
  asdf list all scarb $1 | grep -v "rc" | grep -v "2.8.0"
}

function get_latest_patch_version() {
  get_all_patch_versions $1 | sort -u | tail -1
}

major_minor_versions=($(get_all_patch_versions | cut -d . -f 1,2 | sort -u | tail -3))

scarb_versions=()

if [[ ${major_minor_versions[0]} != "2.6" ]]; then
  scarb_versions+=($(get_latest_patch_version ${major_minor_versions[0]}))
fi

scarb_versions+=($(get_latest_patch_version ${major_minor_versions[1]}))

scarb_versions+=($(get_all_patch_versions ${major_minor_versions[2]}))

echo \"$(echo ${scarb_versions[@]} | sed 's/ /", "/g')\"

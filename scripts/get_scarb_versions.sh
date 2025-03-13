#!/bin/bash
set -e

# This script is used to find the following Scarb versions:
# The current major.minor version along with all its patch versions
# The latest patch versions for the two versions preceding the current one

function get_all_patch_versions() {
  asdf list all scarb "$1" | grep -v "rc"
}

function get_latest_patch_version() {
  get_all_patch_versions "$1" | sort -uV | tail -1
}

major_minor_versions=($(get_all_patch_versions | cut -d . -f 1,2 | sort -uV | tail -3))

declare -a scarb_versions

scarb_versions+=($(get_latest_patch_version "${major_minor_versions[0]}"))
scarb_versions+=($(get_latest_patch_version "${major_minor_versions[1]}"))

scarb_versions+=($(get_all_patch_versions "${major_minor_versions[2]}"))

printf '"%s", ' "${scarb_versions[@]}" | sed 's/, $/\n/'

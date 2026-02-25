#!/bin/bash
set -e

# This script is used to find the following Scarb versions:
# The current major.minor version along with all its patch versions
# The latest patch versions for the two versions preceding the current one
#
# Options:
#   --previous    List only versions older than the current Scarb version (from repo root .tool-versions).

function get_all_patch_versions() {
  asdf list all scarb "$1" | grep -v "rc"
}

function get_latest_patch_version() {
  get_all_patch_versions "$1" | sort -uV | tail -1
}

function version_less_than() {
  _version1="$1"
  _version2="$2"
  printf '%s\n%s' "$_version1" "$_version2" | sort -V | head -n1 | grep -xqvF "$_version2"
}

PREVIOUS_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --previous) PREVIOUS_ONLY=1 ;;
  esac
done

if [[ "$PREVIOUS_ONLY" -eq 1 ]]; then
  repo_root="$(git rev-parse --show-toplevel)"
  tool_versions="$repo_root/.tool-versions"
  if [[ ! -f "$tool_versions" ]]; then
    echo ".tool-versions not found at $tool_versions" >&2
    exit 1
  fi
  current=$(grep -E '^\s*scarb\s+' "$tool_versions" | awk '{ print $2 }')
  if [[ -z "$current" ]]; then
    echo "no scarb version in $tool_versions" >&2
    exit 1
  fi
fi

major_minor_versions=($(get_all_patch_versions | cut -d . -f 1,2 | sort -uV | tail -3))

declare -a scarb_versions

ver=$(get_latest_patch_version "${major_minor_versions[0]}")
if [[ "$PREVIOUS_ONLY" -eq 0 ]] || version_less_than "$ver" "$current"; then
  scarb_versions+=("$ver")
fi
ver=$(get_latest_patch_version "${major_minor_versions[1]}")
if [[ "$PREVIOUS_ONLY" -eq 0 ]] || version_less_than "$ver" "$current"; then
  scarb_versions+=("$ver")
fi
for ver in $(get_all_patch_versions "${major_minor_versions[2]}" | sort -uV); do
  if [[ "$PREVIOUS_ONLY" -eq 0 ]] || version_less_than "$ver" "$current"; then
    scarb_versions+=("$ver")
  fi
done

printf '"%s", ' "${scarb_versions[@]}" | sed 's/, $/\n/'

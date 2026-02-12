#!/bin/sh

# This is internal script used to check correctness of snapshots used in snapshots tests.
# Runs mentioned tests for all Scarb versions from ./get_scarb_versions.sh.
#
# NOTE: this requires ./get_scarb_versions.sh to be present in the CWD.

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BOLD=""
RED=""
YELLOW=""
GREEN=""
RESET=""

# Check whether colors are supported and should be enabled
if [ -z "${NO_COLOR:-}" ] && echo "${TERM:-}" | grep -q "^xterm"; then
  BOLD="\033[1m"
  RED="\033[31m"
  YELLOW="\033[33m"
  GREEN="\033[32m"
  RESET="\033[0m"
fi

SEP="━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

usage() {
  cat <<EOF
Runs e2e snapshot tests (prefixed with \`snap_\`) for each Scarb version.

Usage: $0 [OPTIONS]

Options:
  --fix                  Update snapshots (run with \`INSTA_UPDATE=always\`)
  -h, --help             Print help
EOF
}

run_tests() {
  _fix="${1:-0}"
  if [ "$_fix" = "1" ]; then
    SNFORGE_DETERMINISTIC_OUTPUT=1 INSTA_UPDATE=always cargo test -p forge --test main snap_
  else
    SNFORGE_DETERMINISTIC_OUTPUT=1 cargo test -p forge --test main snap_
  fi
}


main() {
  UPDATE_SNAPSHOTS=0
  while [ $# -gt 0 ]; do
    case "$1" in
      --fix)
        UPDATE_SNAPSHOTS=1;
        shift ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        err "unknown option: $1 (use --help)" ;;
    esac
  done

  need_cmd asdf
  need_cmd cargo

  [ -f "$SCRIPT_DIR/get_scarb_versions.sh" ] || err "get_scarb_versions.sh not found"
  _versions_output=$(cd "$REPO_ROOT" && "$SCRIPT_DIR/get_scarb_versions.sh")
  [ -n "$_versions_output" ] || err "get_scarb_versions.sh produced no output"
  info "Scarb versions: $_versions_output"
  _versions=$(parse_scarb_versions "$_versions_output")
  [ -n "$_versions" ] || err "no scarb versions found"

  cd "$REPO_ROOT" || exit 1

  # Reset Scarb version in `.tool-versions` to original on exit
  _original_scarb=$(asdf current --no-header scarb 2>/dev/null | awk '{print $2}')
  trap cleanup EXIT

  _failed=""
  _ok=0
  _total=0
  for _ver in $_versions; do
    _total=$((_total + 1))
    info "$SEP"
    info "scarb $_ver"
    info "$SEP"
    install_version "scarb" "$_ver"
    set_tool_version "scarb" "$_ver"
    if run_tests "$UPDATE_SNAPSHOTS"; then
      _ok=$((_ok + 1))
    else
      _failed="${_failed}${_failed:+, }$_ver"
    fi
  done

  if [ -n "$_failed" ]; then
    warn "failed: $_failed"
    if [ "$UPDATE_SNAPSHOTS" = "1" ]; then
      exit 0
    fi
    err "run with --fix to update snapshots"
  fi
  info "${GREEN}${_ok}/${_total} passed${RESET}"
}


say() {
  printf 'check_snapshots: %b\n' "$1"
}

info() {
  say "${BOLD}info:${RESET} $1"
}

warn() {
  say "${BOLD}${YELLOW}warn:${RESET} ${YELLOW}$1${RESET}"
}

err() {
  say "${BOLD}${RED}error:${RESET} ${RED}$1${RESET}" >&2
  exit 1
}

need_cmd() {
  if ! check_cmd "$1"; then
    err "need '$1' (command not found)"
  fi
}

check_cmd() {
  command -v "$1" >/dev/null 2>&1
}

ensure() {
  if ! "$@"; then err "command failed: $*"; fi
}

install_version() {
  _tool="$1"
  _installed_version="$2"
  if check_version_installed "$_tool" "$_installed_version"; then
    info "$_tool $_installed_version is already installed"
  else
    info "Installing $_tool $_version..."
    ensure asdf install "$_tool" "$_installed_version"
  fi
}

check_version_installed() {
  _tool="$1"
  _version="$2"
  asdf list "$_tool" | grep -q "^[^0-9]*${_version}$"
}

set_tool_version() {
  _tool="$1"
  _version="$2"
  info "Setting $_tool version to $_version..."
  ensure asdf set "$_tool" "$_version"
}

parse_scarb_versions() {
  echo "$1" | sed 's/"//g' | tr ',' '\n' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$'
}

cleanup() {
  if [ -n "${_original_scarb:-}" ]; then
    asdf set scarb "$_original_scarb" 2>/dev/null || true
  fi
}

main "$@" || exit 1

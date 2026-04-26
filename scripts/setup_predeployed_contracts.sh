#!/usr/bin/env bash

# Generates gzipped predeployed contract artifacts used by cheatnet.
# Clones repositories with source code, adjusts compiler settings,
# builds the contracts, and gzips the generated artifacts in the expected location.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

# starkgate-contracts v3.0.0
STARKGATE_REPO="https://github.com/starknet-io/starkgate-contracts.git"
STARKGATE_REF="07e11c39119a10d5742735be5b1d51894ebf5311"

OUTPUT_DIR="${REPO_ROOT}/crates/cheatnet/src/data/predeployed_contracts"

TMP_DIR=""
cleanup() {
  if [[ -n "${TMP_DIR}" && -d "${TMP_DIR}" ]]; then
    rm -rf "${TMP_DIR}"
  fi
}
trap cleanup EXIT

prepare_starkgate_repo() {
  local repo_url="$1"
  local repo_ref="$2"
  local clone_dir_name="$3"

  TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/${clone_dir_name}.XXXXXX")"
  local repo_dir="${TMP_DIR}/${clone_dir_name}"

  git clone "${repo_url}" "${repo_dir}" >/dev/null 2>&1
  git -C "${repo_dir}" checkout "${repo_ref}" >/dev/null 2>&1

  echo "${repo_dir}"
}

scarb_version_from_tool_versions() {
  local repo_dir="$1"

  awk '$1 == "scarb" { print $2; exit }' "${repo_dir}/.tool-versions" 2>/dev/null || true
}

ensure_scarb_installed() {
  local repo_dir="$1"
  local expected_scarb_version

  expected_scarb_version="$(scarb_version_from_tool_versions "${repo_dir}")"
  if [[ -z "${expected_scarb_version}" ]]; then
    echo "Missing scarb version in ${repo_dir}/.tool-versions" >&2
    exit 1
  fi
}

print_scarb_version_info() {
  local repo_dir="$1"
  local repo_label="$2"
  local repo_ref="$3"
  local expected_scarb_version
  local current_scarb_version

  expected_scarb_version="$(scarb_version_from_tool_versions "${repo_dir}")"
  current_scarb_version="$(
    cd "${repo_dir}"
    scarb --version | awk '{print $2; exit}'
  )"

  echo "Using ${repo_label} at ${repo_ref} with scarb ${current_scarb_version}"

  if [[ -n "${expected_scarb_version}" && "${current_scarb_version}" != "${expected_scarb_version}" ]]; then
    echo "Expected scarb ${expected_scarb_version} from ${repo_dir}/.tool-versions, got ${current_scarb_version}" >&2
    exit 1
  fi
}

ensure_casm_generaton() {
  local file_path="$1"

  if grep -Eq '^\[\[target\.starknet-contract\]\]' "${file_path}"; then
    local tmp_file

    tmp_file="$(mktemp "${TMPDIR:-/tmp}/predeployed-contracts.XXXXXX")"
    awk '
      /^\[\[target\.starknet-contract\]\]$/ {
        print
        print "casm = true"
        next
      }
      $0 != "casm = true" { print }
    ' "${file_path}" > "${tmp_file}"
    mv "${tmp_file}" "${file_path}"
  else
    printf '\n%s\n' "[[target.starknet-contract]]" >> "${file_path}"
    printf '%s\n' "casm = true" >> "${file_path}"
  fi
}

insert_after_header() {
  local file_path="$1"
  local header_regex="$2"
  local inserted_line="$3"
  local tmp_file

  tmp_file="$(mktemp "${TMPDIR:-/tmp}/predeployed-contracts.XXXXXX")"
  awk -v header_regex="${header_regex}" -v inserted_line="${inserted_line}" '
    !inserted && $0 ~ header_regex {
      print
      print inserted_line
      inserted = 1
      next
    }
    { print }
  ' "${file_path}" > "${tmp_file}"
  mv "${tmp_file}" "${file_path}"
}

ensure_debug_info_and_backtrace() {
  local file_path="$1"

  if grep -Eq '^\[profile\.release\.cairo\]' "${file_path}"; then
    if ! grep -Fq "add-statements-code-locations-debug-info = true" "${file_path}"; then
      insert_after_header "${file_path}" '^\[profile\.release\.cairo\]$' 'add-statements-code-locations-debug-info = true'
    fi
    if ! grep -Fq "add-statements-functions-debug-info = true" "${file_path}"; then
      insert_after_header "${file_path}" '^\[profile\.release\.cairo\]$' 'add-statements-functions-debug-info = true'
    fi
    if ! grep -Fq "panic-backtrace = true" "${file_path}"; then
      insert_after_header "${file_path}" '^\[profile\.release\.cairo\]$' 'panic-backtrace = true'
    fi
  else
    cat >> "${file_path}" <<'EOF'

[profile.release.cairo]
add-statements-code-locations-debug-info = true
add-statements-functions-debug-info = true
panic-backtrace = true
EOF
  fi
}

require_file() {
  local file_path="$1"

  if [[ ! -f "${file_path}" ]]; then
    echo "Missing expected artifact: ${file_path}" >&2
    exit 1
  fi
}

gzip_artifacts_from_mappings() {
  local source_root="$1"
  shift

  for artifact_mapping in "$@"; do
    IFS='|' read -r source_relative_path output_relative_path <<< "${artifact_mapping}"
    mkdir -p "${OUTPUT_DIR}/$(dirname "${output_relative_path}")"
  done

  for artifact_mapping in "$@"; do
    IFS='|' read -r source_relative_path output_relative_path <<< "${artifact_mapping}"
    require_file "${source_root}/${source_relative_path}"
  done

  for artifact_mapping in "$@"; do
    IFS='|' read -r source_relative_path output_relative_path <<< "${artifact_mapping}"
    gzip -n -9 -c "${source_root}/${source_relative_path}" > "${OUTPUT_DIR}/${output_relative_path}"
  done
}

prepare_starkgate_contracts() {
  local repo_dir="$1"
  local artifact_mappings=(
    "target/release/strk_ERC20Lockable.compiled_contract_class.json|ERC20Lockable/casm.json.gz"
    "target/release/strk_ERC20Lockable.contract_class.json|ERC20Lockable/sierra.json.gz"
    "target/release/sg_token_ERC20Mintable.compiled_contract_class.json|ERC20Mintable/casm.json.gz"
    "target/release/sg_token_ERC20Mintable.contract_class.json|ERC20Mintable/sierra.json.gz"
  )

  ensure_scarb_installed "${repo_dir}"
  print_scarb_version_info "${repo_dir}" "starkgate-contracts" "${STARKGATE_REF}"

  (
    cd "${repo_dir}"
    ensure_casm_generaton "packages/strk/Scarb.toml"
    ensure_casm_generaton "packages/sg_token/Scarb.toml"
    ensure_debug_info_and_backtrace "Scarb.toml"
    scarb --release build -p strk -p sg_token
  )

  gzip_artifacts_from_mappings "${repo_dir}" "${artifact_mappings[@]}"
}

main() {
  local starkgate_dir
  starkgate_dir="$(prepare_starkgate_repo "${STARKGATE_REPO}" "${STARKGATE_REF}" "starkgate-contracts")"

  prepare_starkgate_contracts "${starkgate_dir}"

  echo "Generated predeployed contract artifacts in ${OUTPUT_DIR}"
}

main "$@"

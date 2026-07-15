#!/usr/bin/env bash

# Checks and optionally updates gas-related expectations in forge integration tests.
# Runs selected tests in a record mode that prints machine-readable actual values,
# then rewrites the matching Rust expectations when invoked with --fix.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BOLD=""
RED=""
YELLOW=""
GREEN=""
RESET=""

if [ -z "${NO_COLOR:-}" ] && echo "${TERM:-}" | grep -q "^xterm"; then
  BOLD="\033[1m"
  RED="\033[31m"
  YELLOW="\033[33m"
  GREEN="\033[32m"
  RESET="\033[0m"
fi

RECORD_PREFIX="SNFORGE_GAS_EXPECTATION"
RECORD_ENV="SNFORGE_GAS_EXPECTATIONS"
RECORD_MODE="record"

TARGET_FILES=(
  "crates/forge/tests/integration/gas.rs"
  "crates/forge/tests/integration/resources.rs"
  "crates/forge/tests/integration/available_gas.rs"
)

TEST_FILTERS=(
  "integration::gas"
  "integration::resources"
  "integration::available_gas"
)

usage() {
  cat <<EOF
Checks forge integration gas expectations.

Usage: $0 [OPTIONS]

Options:
  --fix                  Update gas expectations
  -h, --help             Print help
EOF
}

main() {
  FIX=0
  while [ $# -gt 0 ]; do
    case "$1" in
      --fix)
        FIX=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        err "unknown option: $1 (use --help)"
        ;;
    esac
  done

  need_cmd awk
  need_cmd cargo
  need_cmd cp
  need_cmd diff
  need_cmd grep
  need_cmd mktemp

  cd "$REPO_ROOT"

  TMP_DIR="$(mktemp -d)"
  WORK_ROOT="$TMP_DIR/work"
  RECORDS_FILE="$TMP_DIR/gas-expectation-records.log"
  OUTPUT_FILE="$TMP_DIR/cargo-output.log"
  trap cleanup EXIT

  copy_target_files
  run_record_tests
  collect_records
  apply_records "$WORK_ROOT"

  if has_changes; then
    if [ "$FIX" = "1" ]; then
      copy_updated_files_back
      info "Updated gas expectations"
      ensure cargo fmt
      run_verify_tests
      info "${GREEN}gas expectations updated and verified${RESET}"
    else
      print_changes
      err "gas expectations are stale; run with --fix to update them"
    fi
  elif [ "${RECORD_TESTS_FAILED:-0}" = "1" ]; then
    tail_output
    err "record tests failed, but no gas expectation updates were produced"
  else
    info "${GREEN}gas expectations are up to date${RESET}"
  fi
}

copy_target_files() {
  mkdir -p "$WORK_ROOT"
  for file in "${TARGET_FILES[@]}"; do
    mkdir -p "$WORK_ROOT/$(dirname "$file")"
    cp "$REPO_ROOT/$file" "$WORK_ROOT/$file"
  done
}

run_record_tests() {
  : > "$OUTPUT_FILE"
  RECORD_TESTS_FAILED=0

  for filter in "${TEST_FILTERS[@]}"; do
    info "Recording gas expectations from $filter"
    set +e
    env "$RECORD_ENV=$RECORD_MODE" cargo test -p forge --test main "$filter" -- --nocapture \
      >> "$OUTPUT_FILE" 2>&1
    status=$?
    set -e

    if [ "$status" -ne 0 ]; then
      RECORD_TESTS_FAILED=1
      warn "record test command failed for $filter; continuing to collect emitted records"
    fi
  done
}

collect_records() {
  grep "^$RECORD_PREFIX|" "$OUTPUT_FILE" > "$RECORDS_FILE" || true

  if [ ! -s "$RECORDS_FILE" ]; then
    tail_output
    err "no gas expectation records were emitted"
  fi
}

apply_records() {
  local root="$1"

  while IFS= read -r record; do
    local kind file line test_name
    kind="$(record_field "$record" "kind")"
    file="$(normalize_record_file "$(record_field "$record" "file")")"
    line="$(record_field "$record" "line")"
    test_name="$(record_field "$record" "test")"

    # This helper diagnostic test intentionally uses wrong gas and should not be auto-updated.
    if [ "$test_name" = "gas_assertion_diagnostics" ]; then
      continue
    fi

    if ! is_target_file "$file"; then
      continue
    fi

    case "$kind" in
      gas|available_gas)
        local l1_gas l1_data_gas l2_gas
        l1_gas="$(record_field "$record" "l1_gas")"
        l1_data_gas="$(record_field "$record" "l1_data_gas")"
        l2_gas="$(record_field "$record" "l2_gas")"

        update_gas_vector \
          "$root/$file" \
          "$line" \
          "$l1_gas" \
          "$(format_rust_u64 "$l1_gas")" \
          "$l1_data_gas" \
          "$(format_rust_u64 "$l1_data_gas")" \
          "$l2_gas" \
          "$(format_rust_u64 "$l2_gas")"
        ;;
      syscall|builtin)
        local count
        count="$(record_field "$record" "count")"

        update_assertion_count \
          "$root/$file" \
          "$line" \
          "$count" \
          "$(format_rust_u64 "$count")"
        ;;
      *)
        err "unknown gas expectation record kind: $kind"
        ;;
    esac
  done < "$RECORDS_FILE"
}

update_gas_vector() {
  local file="$1"
  local line="$2"
  local l1_gas_raw="$3"
  local l1_gas="$4"
  local l1_data_gas_raw="$5"
  local l1_data_gas="$6"
  local l2_gas_raw="$7"
  local l2_gas="$8"
  local tmp
  tmp="$(mktemp)"

  awk \
    -v start="$line" \
    -v l1_gas_raw="$l1_gas_raw" \
    -v l1_gas="$l1_gas" \
    -v l1_data_gas_raw="$l1_data_gas_raw" \
    -v l1_data_gas="$l1_data_gas" \
    -v l2_gas_raw="$l2_gas_raw" \
    -v l2_gas="$l2_gas" \
    '
      function normalize(value, normalized) {
        normalized = value
        gsub(/_/, "", normalized)
        return normalized
      }

      function replace_amount(text, key, raw_value, formatted_value, pattern, token, current_value) {
        pattern = key ": GasAmount[(][0-9_]+[)]"

        if (match(text, pattern)) {
          token = substr(text, RSTART, RLENGTH)
          current_value = token
          sub(/^.*GasAmount[(]/, "", current_value)
          sub(/[)].*$/, "", current_value)

          if (normalize(current_value) != normalize(raw_value)) {
            sub(pattern, key ": GasAmount(" formatted_value ")", text)
          }
        }

        return text
      }

      NR >= start && !done {
        if ($0 ~ /GasVector[[:space:]]*\{/) {
          in_vector = 1
        }

        if (in_vector) {
          $0 = replace_amount($0, "l1_gas", l1_gas_raw, l1_gas)
          $0 = replace_amount($0, "l1_data_gas", l1_data_gas_raw, l1_data_gas)
          $0 = replace_amount($0, "l2_gas", l2_gas_raw, l2_gas)

          if ($0 ~ /^[[:space:]]*}[,)]?/) {
            done = 1
            in_vector = 0
          }
        }
      }

      { print }

      END {
        if (!done) {
          exit 42
        }
      }
    ' "$file" > "$tmp" || err "failed to update GasVector near $file:$line"

  mv "$tmp" "$file"
}

update_assertion_count() {
  local file="$1"
  local line="$2"
  local count_raw="$3"
  local count="$4"
  local tmp
  tmp="$(mktemp)"

  awk \
    -v start="$line" \
    -v count_raw="$count_raw" \
    -v count="$count" \
    '
      function normalize(value, normalized) {
        normalized = value
        gsub(/_/, "", normalized)
        return normalized
      }

      function replace_inline_count(text, raw_value, formatted_value, pattern, token, current_value) {
        pattern = ",[[:space:]]*[0-9_]+[[:space:]]*[)][;]"

        if (match(text, pattern)) {
          token = substr(text, RSTART, RLENGTH)
          current_value = token
          gsub(/[^0-9_]/, "", current_value)

          if (normalize(current_value) != normalize(raw_value)) {
            sub(pattern, ", " formatted_value ");", text)
          }
        }

        return text
      }

      function replace_line_count(text, raw_value, formatted_value, current_value) {
        current_value = text
        gsub(/[^0-9_]/, "", current_value)

        if (normalize(current_value) != normalize(raw_value)) {
          sub(/[0-9_]+/, formatted_value, text)
        }

        return text
      }

      NR >= start && !done {
        if ($0 ~ /assert_(syscall|builtin)[(]/ && $0 ~ /[)][;]/) {
          $0 = replace_inline_count($0, count_raw, count)
          done = 1
        } else if ($0 ~ /^[[:space:]]*[0-9_]+[[:space:]]*,[[:space:]]*$/) {
          $0 = replace_line_count($0, count_raw, count)
          done = 1
        }
      }

      { print }

      END {
        if (!done) {
          exit 42
        }
      }
    ' "$file" > "$tmp" || err "failed to update assertion count near $file:$line"

  mv "$tmp" "$file"
}

has_changes() {
  for file in "${TARGET_FILES[@]}"; do
    if ! diff -q "$REPO_ROOT/$file" "$WORK_ROOT/$file" >/dev/null; then
      return 0
    fi
  done

  return 1
}

print_changes() {
  for file in "${TARGET_FILES[@]}"; do
    diff -u "$REPO_ROOT/$file" "$WORK_ROOT/$file" || true
  done
}

copy_updated_files_back() {
  for file in "${TARGET_FILES[@]}"; do
    cp "$WORK_ROOT/$file" "$REPO_ROOT/$file"
  done
}

run_verify_tests() {
  for filter in "${TEST_FILTERS[@]}"; do
    info "Verifying $filter"
    ensure cargo test -p forge --test main "$filter"
  done
}

record_field() {
  local record="$1"
  local key="$2"

  printf '%s\n' "$record" \
    | tr '|' '\n' \
    | sed -n "s/^$key=//p" \
    | tail -n 1
}

normalize_record_file() {
  local file="$1"

  case "$file" in
    "$REPO_ROOT"/*)
      printf '%s\n' "${file#"$REPO_ROOT"/}"
      ;;
    ./*)
      printf '%s\n' "${file#./}"
      ;;
    *)
      printf '%s\n' "$file"
      ;;
  esac
}

is_target_file() {
  local file="$1"

  for target in "${TARGET_FILES[@]}"; do
    if [ "$file" = "$target" ]; then
      return 0
    fi
  done

  return 1
}

format_rust_u64() {
  local value="$1"
  local rest="$value"
  local result=""

  if [ "${#value}" -le 5 ]; then
    printf '%s\n' "$value"
    return
  fi

  while [ "${#rest}" -gt 3 ]; do
    result="_${rest: -3}${result}"
    rest="${rest:0:${#rest}-3}"
  done

  printf '%s%s\n' "$rest" "$result"
}

tail_output() {
  if [ -f "$OUTPUT_FILE" ]; then
    warn "last cargo output lines:"
    tail -n 80 "$OUTPUT_FILE" >&2 || true
  fi
}

say() {
  printf 'check_gas_expectations: %b\n' "$1"
}

info() {
  say "${BOLD}info:${RESET} $1"
}

warn() {
  say "${BOLD}${YELLOW}warn:${RESET} ${YELLOW}$1${RESET}" >&2
}

err() {
  say "${BOLD}${RED}error:${RESET} ${RED}$1${RESET}" >&2
  exit 1
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    err "need '$1' (command not found)"
  fi
}

ensure() {
  if ! "$@"; then
    err "command failed: $*"
  fi
}

cleanup() {
  if [ -n "${TMP_DIR:-}" ]; then
    rm -rf "$TMP_DIR"
  fi
}

main "$@"

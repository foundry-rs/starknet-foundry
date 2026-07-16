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

  # Records were emitted but none matched a target file. This almost always means the emitted
  # `file=` paths don't line up with TARGET_FILES (e.g. a path-format change), which would otherwise
  # silently look like "up to date" while expectations are actually stale. Fail loudly instead.
  if [ "$APPLIED_COUNT" -eq 0 ]; then
    tail_output
    err "collected gas expectation records, but none matched the target files (see TARGET_FILES)"
  fi

  if [ -n "$APPLY_FAILURES" ]; then
    warn "some records could not be applied:"
    printf '%b\n' "$APPLY_FAILURES" >&2
  fi

  if has_changes; then
    if [ "$FIX" = "1" ]; then
      # Persist the updates we could apply before reporting any failures, so progress is never lost.
      copy_updated_files_back
      info "Updated gas expectations"
      ensure cargo fmt
      if [ -n "$APPLY_FAILURES" ]; then
        err "applied available updates, but some records could not be applied (see above); re-run to finish"
      fi
      run_verify_tests
      info "${GREEN}gas expectations updated and verified${RESET}"
    else
      print_changes
      err "gas expectations are stale; run with --fix to update them"
    fi
  elif [ -n "$APPLY_FAILURES" ]; then
    err "no expectation values changed, but some records could not be applied (see above)"
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

# Runs the target test suites with recording enabled (`SNFORGE_GAS_EXPECTATIONS=record`), appending
# all output to `$OUTPUT_FILE`. `--nocapture` is required so the records printed by the assertions
# reach the log instead of being swallowed by libtest.
#
# Test failures are tolerated on purpose: when expectations are stale the assertions still fail, but
# the actual values are emitted before that happens, so we want to keep the output and carry on.
# `RECORD_TESTS_FAILED` merely notes that a failure occurred, for `main` to interpret later.
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

# Extracts the machine-readable records emitted during recording out of the raw cargo output into
# `$RECORDS_FILE`, and fails if none were found (which would mean recording is broken).
collect_records() {
  # Extract from the prefix to end-of-line rather than anchoring with `^`, so a record still gets
  # picked up if another (parallel) test happens to print onto the same line before it.
  grep -o "$RECORD_PREFIX|.*" "$OUTPUT_FILE" > "$RECORDS_FILE" || true

  if [ ! -s "$RECORDS_FILE" ]; then
    tail_output
    err "no gas expectation records were emitted"
  fi
}

# Applies every collected record to the copies under `$root`. Records are keyed by file:line from
# the original sources, and each update is line-preserving (in-place `sub`, no lines added/removed),
# so line numbers stay valid across multiple records touching the same file.
#
# Sets two globals inspected by `main`:
#   APPLIED_COUNT   - number of records that matched a target file (whether or not the value changed)
#   APPLY_FAILURES  - newline-separated list of records that matched a target but could not be applied
apply_records() {
  local root="$1"
  APPLIED_COUNT=0
  APPLY_FAILURES=""

  while IFS= read -r record; do
    local kind file line target
    kind="$(record_field "$record" "kind")"
    file="$(record_field "$record" "file")"
    line="$(record_field "$record" "line")"

    # Skip records emitted from files we don't manage (matched by path suffix, so the exact path
    # format emitted by `Location::caller()` doesn't matter).
    if ! target="$(resolve_target_file "$file")"; then
      continue
    fi

    APPLIED_COUNT=$((APPLIED_COUNT + 1))

    case "$kind" in
      gas|available_gas)
        if ! update_gas_vector \
          "$root/$target" \
          "$line" \
          "$(format_rust_u64 "$(record_field "$record" "l1_gas")")" \
          "$(format_rust_u64 "$(record_field "$record" "l1_data_gas")")" \
          "$(format_rust_u64 "$(record_field "$record" "l2_gas")")"; then
          record_apply_failure "GasVector near $target:$line"
        fi
        ;;
      syscall|builtin)
        if ! update_assertion_count \
          "$root/$target" \
          "$line" \
          "$(format_rust_u64 "$(record_field "$record" "count")")"; then
          record_apply_failure "$kind count near $target:$line"
        fi
        ;;
      *)
        err "unknown gas expectation record kind: $kind"
        ;;
    esac
  done < "$RECORDS_FILE"
}

record_apply_failure() {
  APPLY_FAILURES="${APPLY_FAILURES}${APPLY_FAILURES:+
}  - $1"
}

# Rewrites the `GasVector { .. }` literal that starts at (or just below) `$file:$line` so its
# `l1_gas` / `l1_data_gas` / `l2_gas` amounts match the recorded values.
#
# Values are already Rust-formatted (e.g. `440_000`); comparison ignores `_` separators, so an
# amount is only rewritten when it actually differs (no spurious diffs). Returns non-zero if the
# literal can't be located.
#
# Args: file line l1_gas l1_data_gas l2_gas
update_gas_vector() {
  local file="$1"
  local line="$2"
  local l1_gas="$3"
  local l1_data_gas="$4"
  local l2_gas="$5"
  local tmp
  tmp="$(mktemp)"

  awk \
    -v start="$line" \
    -v l1_gas="$l1_gas" \
    -v l1_data_gas="$l1_data_gas" \
    -v l2_gas="$l2_gas" \
    '
      function normalize(value, normalized) {
        normalized = value
        gsub(/_/, "", normalized)
        return normalized
      }

      function replace_amount(text, key, value, pattern, token, current_value) {
        pattern = key ": GasAmount[(][0-9_]+[)]"

        if (match(text, pattern)) {
          token = substr(text, RSTART, RLENGTH)
          current_value = token
          sub(/^.*GasAmount[(]/, "", current_value)
          sub(/[)].*$/, "", current_value)

          if (normalize(current_value) != normalize(value)) {
            sub(pattern, key ": GasAmount(" value ")", text)
          }
        }

        return text
      }

      NR >= start && !done {
        if ($0 ~ /GasVector[[:space:]]*\{/) {
          in_vector = 1
        }

        if (in_vector) {
          $0 = replace_amount($0, "l1_gas", l1_gas)
          $0 = replace_amount($0, "l1_data_gas", l1_data_gas)
          $0 = replace_amount($0, "l2_gas", l2_gas)

          # Match a closing brace anywhere on the line, so both multi-line literals (`}` on its own
          # line) and single-line literals (`GasVector { ... }`) terminate the block.
          if ($0 ~ /}/) {
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
    ' "$file" > "$tmp" || { rm -f "$tmp"; return 1; }

  mv "$tmp" "$file"
}

# Rewrites the expected count of an `assert_syscall` / `assert_builtin` call located at (or just
# below) `$file:$line` to the recorded value. Handles both the single-line form (`assert_syscall(.., N);`)
# and the multi-line form (the count sits on its own line as `N,`).
#
# The count is already Rust-formatted; comparison ignores `_` separators so it is only rewritten
# when it actually differs, and it returns non-zero if the call can't be located.
#
# Args: file line count
update_assertion_count() {
  local file="$1"
  local line="$2"
  local count="$3"
  local tmp
  tmp="$(mktemp)"

  awk \
    -v start="$line" \
    -v count="$count" \
    '
      function normalize(value, normalized) {
        normalized = value
        gsub(/_/, "", normalized)
        return normalized
      }

      function replace_inline_count(text, value, pattern, token, current_value) {
        pattern = ",[[:space:]]*[0-9_]+[[:space:]]*[)][;]"

        if (match(text, pattern)) {
          token = substr(text, RSTART, RLENGTH)
          current_value = token
          gsub(/[^0-9_]/, "", current_value)

          if (normalize(current_value) != normalize(value)) {
            sub(pattern, ", " value ");", text)
          }
        }

        return text
      }

      function replace_line_count(text, value, current_value) {
        current_value = text
        gsub(/[^0-9_]/, "", current_value)

        if (normalize(current_value) != normalize(value)) {
          sub(/[0-9_]+/, value, text)
        }

        return text
      }

      NR >= start && !done {
        if ($0 ~ /assert_(syscall|builtin)[(]/ && $0 ~ /[)][;]/) {
          $0 = replace_inline_count($0, count)
          done = 1
        } else if ($0 ~ /^[[:space:]]*[0-9_]+[[:space:]]*,[[:space:]]*$/) {
          $0 = replace_line_count($0, count)
          done = 1
        }
      }

      { print }

      END {
        if (!done) {
          exit 42
        }
      }
    ' "$file" > "$tmp" || { rm -f "$tmp"; return 1; }

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

# Resolves a record's file path to the matching entry in TARGET_FILES and prints it, or returns 1.
# Matches on path suffix in either direction, so it works regardless of whether
# `Location::caller()` emits a workspace-relative, package-relative or absolute path.
resolve_target_file() {
  local file="$1"
  local target

  for target in "${TARGET_FILES[@]}"; do
    if [ "$file" = "$target" ]; then
      printf '%s\n' "$target"
      return 0
    fi
    case "$file" in
      */"$target")
        printf '%s\n' "$target"
        return 0
        ;;
    esac
    case "$target" in
      */"$file")
        printf '%s\n' "$target"
        return 0
        ;;
    esac
  done

  return 1
}

# Formats a numeric literal the way it is written in the sources: digits grouped in threes with `_`
# separators (e.g. 440000 -> 440_000). Values with 5 or fewer digits are left as-is, matching the
# existing convention in the test files (e.g. 40000 stays 40000).
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

---
name: update-gas-expectations
description: >-
  Bulk-refresh the hardcoded gas / resource expectations in Starknet Foundry's Forge integration
  tests (assert_gas, assert_syscall, assert_builtin, and the available_gas error strings). Use this
  whenever those tests fail with expected-vs-actual mismatches after a Scarb or blockifier bump, or
  whenever the user asks to "update gas values", "fix the gas tests", "regenerate gas expectations",
  or mentions that gas.rs / resources.rs / available_gas.rs assertions are stale. Prefer this skill
  over manually editing the numbers one by one — it drives the tests and rewrites every stale value
  in one guided pass.
---

# Update gas expectations

## Objective

Many Forge integration tests assert on exact gas usage and resource counts. Every Scarb or
blockifier bump can change the underlying gas model, so the expected values in the tests can go stale.
Updating them by hand is slow and error-prone.

This skill automates that: run the relevant tests, read the actual value each failing assertion
reports, and rewrite the matching expectation in the source. The assertion helpers are written to
print `expected:` and `actual:` on failure (see `crates/forge/tests/utils/runner.rs`), so the actual
value is always available in the panic message — you rewrite the source to match it, then re-run
until everything is green.

The loop converges because each run surfaces the actual values of the assertions that failed; fixing
them and re-running surfaces the next batch (a single test function has several assertions, and only
the first failing one panics per run).

## Scope

Edit only expectation literals and directly related calculation comments in:

- `crates/forge/tests/integration/gas.rs` — `assert_gas` calls (`GasVector { l1_gas, l1_data_gas, l2_gas }`)
- `crates/forge/tests/integration/resources.rs` — `assert_syscall` / `assert_builtin` counts
- `crates/forge/tests/integration/available_gas.rs` — the `l2_gas: ~NNNNNN` number inside the expected error string

Do not modify:
- assertion logic;
- test bodies (the Cairo source inside `test_case!`);
- production code;
- files outside this list.

You are only updating numeric expectations and comments that explain those exact expectations.

## Repository and instruction precedence

When instructions conflict, use the following precedence:

1. explicit user requirements;
2. repository-level instructions such as `AGENTS.md`;
3. instructions located closest to the modified file;
4. existing conventions in the target package;
5. this skill's defaults.

## Prerequisites

- `scarb` must be installed at the version pinned in `.tool-versions` (run `asdf install` if `asdf` is
  used). Without it the tests fail on `scarb --version` before any gas assertion runs — that is an
  environment problem, not a stale expectation. If you see `Command scarb failed`, inspect the error;
  do not edit anything.
- Run from the repo root.

### Existing working-tree changes

Before editing, inspect existing changes in the target files:

```sh
git status --short -- \
  crates/forge/tests/integration/gas.rs \
  crates/forge/tests/integration/resources.rs \
  crates/forge/tests/integration/available_gas.rs

git diff -- \
  crates/forge/tests/integration/gas.rs \
  crates/forge/tests/integration/resources.rs \
  crates/forge/tests/integration/available_gas.rs
```

Preserve all pre-existing user changes.

Never:

- revert unrelated changes;
- replace an entire file when a narrow edit is sufficient;
- format or rewrite surrounding code unnecessarily;
- assume that every existing change was produced by this skill.

## Workflow

Work one target file at a time (`gas.rs`, then `resources.rs`, then `available_gas.rs`). For each:

### 1. Run the tests with EXACT assertions

Do not pass the `non_exact_gas_assertions` feature — the margin mode hides small diffs, and you
want precise target values.

```sh
cargo test -p forge --test main integration::gas
cargo test -p forge --test main integration::resources
cargo test -p forge --test main integration::available_gas
```

If nothing fails, that file's expectations are already up to date — move on.

### 2. For each failure, read the reported actual value

The panic messages look like this:

`assert_gas`:
```
Gas assertion failed for test case `test_package_integrationtest::test_case::some_test`.
expected: l1_gas: 0, l1_data_gas: 0, l2_gas: 40000
actual:   l1_gas: 0, l1_data_gas: 0, l2_gas: 42000
diff:     l1_gas: 0, l1_data_gas: 0, l2_gas: 2000
```

`assert_syscall` / `assert_builtin`:
```
Syscall assertion failed for test case `test_package_integrationtest::test_case::keccak` (syscall `Keccak`).
expected: 1
actual:   2
```
```
Builtin assertion failed for test case `test_package_integrationtest::test_case::range_check` (builtin `range_check`).
expected: 3
actual:   4
```

The `actual:` line is the value compared by the assertion helper after any helper-side
normalization. For `assert_syscall` and non-`range_check` `assert_builtin` calls, write it verbatim.
For `assert_builtin(..., BuiltinName::range_check, N)`, the helper subtracts 1 before comparing, so
write `actual + 1` in the source.

### 3. Rewrite the matching expectation

Use the full test case path from the message to identify the exact Cairo test function and matching
Rust assertion. Helper calls may still pass only the final path segment, such as `"some_test"`; use
that short argument only after the full path has disambiguated the case. Then replace the numbers
according to the assertion type:

- `assert_gas`: update the three `GasAmount(...)` values in the `GasVector { .. }` literal.
- `assert_syscall` and non-`range_check` `assert_builtin`: update the last numeric argument to
  `actual`.
- `assert_builtin` with `BuiltinName::range_check`: update the last numeric argument to `actual + 1`.
- `available_gas.rs`: update the number after `l2_gas: ~` inside the expected error string.

If nearby comments explain the expected value, update them in the same pass. This is especially
important for comments that derive syscall or builtin gas from Blockifier versioned constants, for
example `n_steps * step_gas_cost + sum(builtin_count * builtin_gas_cost)`. Use the versioned
constants file referenced by the test comments, or the current Blockifier version from `Cargo.lock`
when the comments are stale after a dependency bump. Update the formula inputs, computed totals,
Blockifier/versioned-constants version labels, and source links so the comments justify the new
expectation. If the expectation can be updated from `actual:` but the matching calculation is not
clear from versioned constants, update the expectation and flag the stale/unclear comment instead of
inventing a formula.

Example — `assert_gas`:
```rust
// before (from the message above: test case `test_package_integrationtest::test_case::some_test`,
// actual l2_gas: 42000)
assert_gas(
    &result,
    "some_test",
    GasVector {
        l1_gas: GasAmount(0),
        l1_data_gas: GasAmount(0),
        l2_gas: GasAmount(40000),   // <- 42000
    },
);
```

Example — `assert_syscall`:
```rust
assert_syscall(&result, "keccak", SyscallSelector::Keccak, 1);   // <- 2
```

### 4. Re-run and repeat

Re-run the same test command. Each pass fixes the assertions that panicked; keep looping until the
file is fully green, then move to the next target file.

### 5. Final validation

After all three filtered suites pass, run:

```sh
cargo fmt --check

cargo test -p forge --test main integration::gas -- --test-threads=1
cargo test -p forge --test main integration::resources -- --test-threads=1
cargo test -p forge --test main integration::available_gas -- --test-threads=1
```

Then show the user `git diff` of the three files so they can review before committing.

## Do NOT touch the assertion-helper diagnostics tests

Some assertion-helper tests deliberately assert on wrong values to verify the failure output itself.
Never "fix" these — they are supposed to fail-then-be-caught internally:

- In `crates/forge/tests/utils/runner.rs`: `assert_gas_failure_shows_gas_diff_and_test_case_name`,
  `assert_gas_reports_when_test_case_is_missing`, `assert_gas_rejects_fuzzing_test_case`,
  `assert_gas_reports_non_passed_test_case`.

These construct expectations inline to exercise the panic path; if you see a `GasVector { GasAmount(1),
GasAmount(2), GasAmount(3) }` or `GasVector::default()` inside a test whose *purpose* is checking the
message, leave it alone.

## Guardrails

- If a test fails for any reason other than an expected-vs-actual value mismatch (compile error,
  contract deploy failure, `scarb` missing, a genuine logic panic), stop and report it. Do not paper
  over real breakage by editing numbers.
- Only change numeric literals / the error-string number. If an expectation is written in a shape you
  can't confidently map to the reported value, flag it for the user instead of guessing.
- Keep going until the full suite for the three files is green — a partial update leaves the tests red.

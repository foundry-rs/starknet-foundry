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

## What this does and why

Many Forge integration tests assert on **exact** gas usage and resource counts. Every Scarb or
blockifier bump changes the underlying gas model, so dozens of hardcoded numbers go stale at once.
Updating them by hand is slow and error-prone.

This skill automates that: run the relevant tests, read the *actual* value each failing assertion
reports, and rewrite the matching expectation in the source. The assertion helpers are written to
print `expected:` and `actual:` on failure (see `crates/forge/tests/utils/runner.rs`), so the actual
value is always available in the panic message — you rewrite the source to match it, then re-run
until everything is green.

The loop converges because each run surfaces the actual values of the assertions that failed; fixing
them and re-running surfaces the next batch (a single test function has several assertions, and only
the first failing one panics per run).

## Scope — only touch these files

Edit **only** expectation literals in:

- `crates/forge/tests/integration/gas.rs` — `assert_gas` calls (`GasVector { l1_gas, l1_data_gas, l2_gas }`)
- `crates/forge/tests/integration/resources.rs` — `assert_syscall` / `assert_builtin` counts
- `crates/forge/tests/integration/available_gas.rs` — the `l2_gas: ~NNNNNN` number inside the expected error string

Never change assertion *logic*, test bodies (the Cairo source inside `test_case!`), production code, or
files outside this list. You are only updating numeric expectations.

## Prerequisites

- `scarb` must be installed at the version pinned in `.tool-versions` (run `asdf install` if `asdf` is
  used). Without it the tests fail on `scarb --version` before any gas assertion runs — that is an
  environment problem, not a stale expectation. If you see `Command scarb failed`, stop and tell the
  user to install scarb; do not edit anything.
- Run from the repo root.

## Workflow

Work one target file at a time (`gas.rs`, then `resources.rs`, then `available_gas.rs`). For each:

### 1. Run the tests with EXACT assertions

Do **not** pass the `non_exact_gas_assertions` feature — the margin mode hides small diffs, and you
want precise target values.

```sh
cargo test -p forge --test main integration::gas
cargo test -p forge --test main integration::resources
cargo test -p forge --test main integration::available_gas
```

If nothing fails, that file's expectations are already up to date — move on.

### 2. For each failure, read the reported actual value

The panic messages look like this:

**`assert_gas`:**
```
Gas assertion failed for test case `some_test`.
expected: l1_gas: 0, l1_data_gas: 0, l2_gas: 40000
actual:   l1_gas: 0, l1_data_gas: 0, l2_gas: 42000
diff:     l1_gas: 0, l1_data_gas: 0, l2_gas: 2000
```

**`assert_syscall` / `assert_builtin`:**
```
Syscall assertion failed for test case `keccak` (syscall `Keccak`).
expected: 1
actual:   2
```
```
Builtin assertion failed for test case `range_check` (builtin `RangeCheck`).
expected: 4
actual:   4
```

The `actual:` line is already normalized to the value the source should hold — including the
`range_check` off-by-one adjustment — so you write it verbatim, no arithmetic.

### 3. Rewrite the matching expectation

Locate the assertion by the **test case name** from the message (it is the string argument to the
helper), then replace the numbers with the `actual:` values.

- `assert_gas`: update the three `GasAmount(...)` values in the `GasVector { .. }` literal.
- `assert_syscall` / `assert_builtin`: update the last numeric argument.
- `available_gas.rs`: update the number after `l2_gas: ~` inside the expected error string.

**Example — `assert_gas`:**
```rust
// before (from the message above: test case `some_test`, actual l2_gas: 42000)
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

**Example — `assert_syscall`:**
```rust
assert_syscall(&result, "keccak", SyscallSelector::Keccak, 1);   // <- 2
```

### 4. Re-run and repeat

Re-run the same test command. Each pass fixes the assertions that panicked; keep looping until the
file is fully green, then move to the next target file.

### 5. Finalize

After all three files pass:

```sh
cargo fmt
cargo test -p forge --test main integration::gas
cargo test -p forge --test main integration::resources
cargo test -p forge --test main integration::available_gas
```

Then show the user `git diff` of the three files so they can review before committing (typically the
diff is committed together with the Scarb/blockifier bump).

## Do NOT touch the diagnostics tests

Some tests deliberately assert on **wrong** values to verify the failure output itself. Never "fix"
these — they are supposed to fail-then-be-caught internally:

- In `gas.rs`: `assert_gas_failure_shows_gas_diff_and_test_case_name`,
  `assert_gas_reports_when_test_case_is_missing`, `assert_gas_rejects_fuzzing_test_case`,
  `assert_gas_reports_non_passed_test_case`.

These construct expectations inline to exercise the panic path; if you see a `GasVector { GasAmount(1),
GasAmount(2), GasAmount(3) }` or `GasVector::default()` inside a test whose *purpose* is checking the
message, leave it alone.

## Guardrails

- If a test fails for any reason **other** than an expected-vs-actual value mismatch (compile error,
  contract deploy failure, `scarb` missing, a genuine logic panic), stop and report it. Do not paper
  over real breakage by editing numbers.
- Only change numeric literals / the error-string number. If an expectation is written in a shape you
  can't confidently map to the reported value, flag it for the user instead of guessing.
- Keep going until the full suite for the three files is green — a partial update leaves the tests red.

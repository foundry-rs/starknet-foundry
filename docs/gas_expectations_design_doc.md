# Bulk-updating gas test expectations

## Rationale of the solution

Many Forge integration tests assert on **exact** gas-related values: the consumed `GasVector`
(`assert_gas`) and the number of syscalls/builtins used (`assert_syscall`, `assert_builtin`). There
are dozens of these values hardcoded across `gas.rs` and `resources.rs`.

The problem: every Scarb or blockifier bump changes the gas model, so all of these values have to be
updated by hand. This is tedious, error-prone and discourages doing bumps.

Goal: a tool, in the spirit of the existing `scripts/check_snapshots.sh`, that:
- in **check** mode reports whether the expectations are up to date (useful in CI / locally),
- in **`--fix`** mode bulk-updates them to the actual values.

Alternatives considered earlier (migrating to `insta`/JSON snapshots) would change the shape of the
assertions and hurt the readability of the gas tests. The chosen solution keeps the assertions in
their current, explicit form and only automates updating the numbers.

## User perspective description of the solution

For a contributor the API is a single script:

```sh
# check whether gas expectations are up to date (prints a diff and errors out if not)
./scripts/check_gas_expectations.sh

# update them in place, format and verify
./scripts/check_gas_expectations.sh --fix
```

Typical flow: after a Scarb/blockifier bump run `--fix`, review the generated diff and commit it
together with the bump. The way tests are written does not change — assertions look exactly as
before. Details live in the developer docs ("Gas expectations" page).

## Technical overview of the solution

The mechanism consists of **two cooperating halves** — Rust code (recording) and a Bash script
(rewriting).

**Mental model:** "Rust knows the actual value and the exact source line its expectation lives on;
Bash just surgically rewrites that line."

1. **Recording (Rust).** The assertion helpers are marked `#[track_caller]`. When
   `SNFORGE_GAS_EXPECTATIONS=record` is set, each of them prints a machine-readable record to stderr:
   the `file:line` of the call site (via `Location::caller()`), the actual values, and the kind
   (`gas` / `syscall` / `builtin`).

2. **Rewriting (Bash + awk).** The script runs the selected tests in record mode, collects the
   records, and for each one goes to the recorded `file:line` in a **copy** of the file and rewrites
   the literal to the actual value. Finally it compares the copies against the repo: in check mode it
   prints a diff, in `--fix` it applies the changes, formats and verifies with the tests.

Non-obvious decisions and their motivation:

- **Why `file:line` from `Location::caller()` instead of parsing the AST / test names?** The record
  carries exact coordinates, so Bash/awk don't have to "understand" Rust — they do a local rewrite
  that is robust to changes in test structure.
- **Why don't the count assertions (`syscall`/`builtin`) fail in record mode?** Tests have several
  assertions per function; if the first stale assertion panicked, the rest wouldn't emit their
  records and the update wouldn't converge in a single run.
- **Why work on copies instead of the originals?** Recording must see the original files (so line
  numbers match) while we mutate copies — the original is untouched until `--fix`. All rewrites are
  line-preserving, so multiple records targeting one file stay consistent.
- **Why "suffix" file matching?** `Location::caller().file()` may return a path in different formats
  (workspace-/package-relative/absolute); suffix matching makes the script independent of that.
  Additionally: if records were collected but none matched a target file, the script fails **loudly**
  instead of falsely reporting "up to date".
- **Partial resilience:** a single record that can't be applied doesn't abort the whole run — the
  other changes are still saved and the failures are reported together.
- **Diagnostics tests** (which deliberately assert on wrong values) are excluded from recording via a
  thread-local switch, so the script never "fixes" them.

**Scope:** the script only operates on the files listed in `TARGET_FILES` (currently `gas.rs` and
`resources.rs`).

**Open questions:**
- Should the script be wired into CI as a check (currently `check_snapshots.sh` is not), or stay a
  local tool?
- The contract constants (`SNFORGE_GAS_EXPECTATION*`) are duplicated on the Rust and Bash sides —
  worth centralizing?
- awk fragility against unusual literal formatting (single-line `GasVector` is handled, but other
  formatting variants may need widening the patterns).

## Implementation plan

The feature is already implemented (issue
[#4485](https://github.com/foundry-rs/starknet-foundry/issues/4485), building on
[#4484](https://github.com/foundry-rs/starknet-foundry/issues/4484) — showing expected vs actual in
`assert_gas`). Remaining, optional steps: wiring into CI and centralizing the contract constants.

---

Use :ok_hand: to approve the design doc

Use :thinking_face: if you are reviewing it / want to review but have not approved yet

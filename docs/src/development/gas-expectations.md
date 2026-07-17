# Gas expectations
> 💡 **Info**
> This tutorial is only relevant if you wish to contribute to Starknet Foundry.
> If you plan to only use it as a tool for your project, you can skip this part.

Some Forge integration tests assert on exact gas usage and resource counts, for example:

- `assert_gas` - exact `GasVector` consumed by a test,
- `assert_syscall` / `assert_builtin` - number of syscalls / builtins used,
- the `l2_gas: ~NNNNNN` number inside the expected `available_gas` error strings.

These expected values are hardcoded in the test sources and change whenever the underlying gas
model changes (typically after a Scarb or blockifier bump). Updating them by hand is
tedious and error-prone.

The affected files are:

- `crates/forge/tests/integration/gas.rs`
- `crates/forge/tests/integration/resources.rs`
- `crates/forge/tests/integration/available_gas.rs`

## The `update-gas-expectations` skill

We ship a [Claude Code](https://docs.claude.com/en/docs/claude-code/overview) skill,
`update-gas-expectations`, that automates the refresh. It lives at `.claude/skills/update-gas-expectations/SKILL.md`.

The assertion helpers print both `expected:` and `actual:` on failure, so the actual value is always
available in the panic message. The skill drives the relevant tests, reads the reported `actual:`
value for each failing assertion, and rewrites the matching expectation in the source. It then
re-runs until every assertion is green.

### Usage

From an interactive Claude Code session in the repo root, invoke it explicitly:

```
/update-gas-expectations
```

or simply ask Claude to "update the gas expectations" / "fix the gas tests" - the skill's
description makes it trigger automatically for those requests.

### When to run it

Run it after any change that affects gas computation - most commonly after bumping Scarb or
blockifier - and commit the regenerated expectations together with the bump.

## Notes

- Only numeric expectation literals in the three files listed above are edited. Assertion logic,
  test bodies (the Cairo source inside `test_case!`) and production code are never touched.
- The skill runs the tests with **exact** assertions (without the `non_exact_gas_assertions`
  feature), so the recorded values are precise rather than margin-based.
- Tests that deliberately assert on *wrong* values (the `assert_gas` diagnostics tests, such as
  `assert_gas_failure_shows_gas_diff_and_test_case_name`) are excluded - the skill never "fixes"
  them.
- Any failure that is not an expected-vs-actual mismatch (compile error, contract deploy failure,
  missing `scarb`, a genuine logic panic) causes the skill to stop and report instead of editing
  numbers.

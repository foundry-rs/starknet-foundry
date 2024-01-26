# `snforge test`

Run tests for a project in the current directory.

## `[TEST_FILTER]`

Passing a test filter will only run tests with
an [absolute module tree path](https://book.cairo-lang.org/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#paths-for-referring-to-an-item-in-the-module-tree)
containing this filter.

## `-e`, `--exact`

Will only run a test with a name exactly matching the test filter.
Test filter must be a whole qualified test name e.g. `package_name::my_test` instead of just `my_test`.

## `-x`, `--exit-first`

Stop executing tests after the first failed test.

## `-p`, `--package <SPEC>`

Packages to run this command on, can be a concrete package name (`foobar`) or a prefix glob (`foo*`).

## `-w`, `--workspace`

Run tests for all packages in the workspace.

## `-r`, `--fuzzer-runs` `<FUZZER_RUNS>`

Number of fuzzer runs.

## `-s`, `--fuzzer-seed` `<FUZZER_SEED>`

Seed for the fuzzer.

## `--ignored`

Run only tests marked with `#[ignore]` attribute.

## `--include-ignored`

Run all tests regardless of `#[ignore]` attribute.

## `--rerun-failed`

Run tests that failed during the last run

## `--color` `<WHEN>`

Control when colored output is used. Valid values:
- `auto` (default): automatically detect if color support is available on the terminal. 
- `always`: always display colors.
- `never`: never display colors.

## `--save-trace-data`
Saves execution traces of test cases which have passed and are not fuzz tests to files. Traces can be used for profiling purposes.


## `--use_scarb_collector`

Uses Scarb to find and compile tests. Requires at least Scarb nightly-2023-12-04

> ⚠️ **Warning**
> 
> This is an experimental feature. Some functionalities might not work.

## `-h`, `--help`

Print help.

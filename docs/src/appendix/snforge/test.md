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

## `--detailed-resources`

Display additional info about used resources for passed tests.

## `--save-trace-data`

Saves execution traces of test cases which pass and are not fuzz tests. You can use traces for profiling purposes.

## `--build-profile`

Saves trace data and then builds profiles of test cases which pass and are not fuzz tests. 
You need [cairo-profiler](https://github.com/software-mansion/cairo-profiler) installed on your system. You can set a custom path to cairo-profiler with `CAIRO_PROFILER` env variable. Profile can be read with pprof, more information: [cairo-profiler](https://github.com/software-mansion/cairo-profiler), [pprof](https://github.com/google/pprof?tab=readme-ov-file#building-pprof)

## `--max-n-steps` `<MAX_N_STEPS>`

Number of maximum steps during a single test. For fuzz tests this value is applied to each subtest separately.

## `-h`, `--help`

Print help.

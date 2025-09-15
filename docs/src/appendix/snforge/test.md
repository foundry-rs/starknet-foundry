# `snforge test`

Run tests for a project in the current directory.

## `[TEST_FILTER]`

Passing a test filter will only run tests with
an [absolute module tree path](https://book.cairo-lang.org/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#paths-for-referring-to-an-item-in-the-module-tree)
containing this filter.

## `--trace-verbosity <TRACE_VERBOSITY>`

Sets the level of detail shown in execution traces.

Valid values:

- `minimal`: Only test name, contract name, and selector
- `standard`: Includes calldata and call result
- `detailed`: Full trace, including nested calls, caller address, and panic reasons

## `--trace-components <TRACE_COMPONENTS>...`

Selects specific trace elements to include in the execution flow output.

Available components:

- `contract-name`
- `entry-point-type`
- `calldata`
- `contract-address`
- `caller-address`
- `call-type`
- `call-result`

## `-e`, `--exact`

Will only run a test with a name exactly matching the test filter.
Test filter must be a whole qualified test name e.g. `package_name::my_test` instead of just `my_test`.

## `--skip <SKIP>`

Skips any tests whose name contains the given `SKIP` string

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

## `--coverage`

Saves trace data and then generates coverage report of test cases which pass and are not fuzz tests.
You need [cairo-coverage](https://github.com/software-mansion/cairo-coverage) installed on your system. You can set a custom path to cairo-coverage with `CAIRO_COVERAGE` env variable.

## `--max-n-steps` `<MAX_N_STEPS>`

Number of maximum steps during a single test. For fuzz tests this value is applied to each subtest separately.

##  `-F`, `--features` `<FEATURES>`
Comma separated list of features to activate.

## `--all-features`
Activate all available features.

## `--no-default-features`
Do not activate the `default` feature.

## `--no-optimization`
Build contract artifacts in a separate [starknet contract target](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html#starknet-contract-target).
Enabling this flag will slow down the compilation process, but the built contracts will more closely resemble the ones used on real networks. This is set to `true` when using Scarb version less than `2.8.3`.

## `--tracked-resource`

Set tracked resource for test execution. Impacts overall test gas cost. Valid values:
- `cairo-steps` (default): track cairo steps, uses vm `ExecutionResources` (steps, builtins, memory holes) to describe  resources consumed by the test.
- `sierra-gas` (sierra 1.7.0+ is required): track sierra gas, uses cairo native `CallExecution` (sierra gas consumption) to describe computation resources consumed by the test.
To learn more about fee calculation formula (and an impact of tracking sierra gas on it) please consult [starknet docs](https://docs.starknet.io/architecture-and-concepts/fees/#overall_fee)

##  `-P`, `--profile` `<PROFILE>`
Specify the profile to use by name.

## `--release`
Use Scarb release profile.

## `--dev`
Use Scarb dev profile.

## `--experimental-oracles`

Enable experimental [oracles](../../snforge-advanced-features/oracles.md) support.

## `-h`, `--help`

Print help.

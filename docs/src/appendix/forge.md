# `snforge`

Run forge in the current directory

## `[TEST_FILTER]`

Passing a test filter will only run tests with an [absolute module tree path](https://book.cairo-lang.org/ch06-03-paths-for-referring-to-an-item-in-the-module-tree.html?highlight=path#paths-for-referring-to-an-item-in-the-module-tree)
containing this filter.

## `-e`, `--exact`

Will only run a test with a name exactly matching the test filter.
Test filter must be a whole qualified test name e.g. `package_name::my_test` instead of just `my_test`.

## `--init <NAME>`

 Create a new directory and forge project named `<NAME>`.

## `-x`, `--exit-first`

Stop executing tests after the first failed test.

## `-p`, `--package`

Packages to run this command on, can be a concrete package name (`foobar`) or a prefix glob (`foo*`).

## `-w`, `--workspace`

Run tests for all packages in the workspace.

## `-r`, `--fuzzer-runs` `<FUZZER_RUNS>`  

Number of fuzzer runs.

##  `-s`, `--fuzzer-seed` `<FUZZER_SEED>`  

Seed for the fuzzer.

## `-h`, `--help`

Print help.

## `-V`, `--version`

Print version.

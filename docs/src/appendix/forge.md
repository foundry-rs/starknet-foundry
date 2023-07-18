# `forge [test filter]`

Run forge in the current directory

## `[test filter]`

Passing a test filter will only run tests with a name containing this filter.

## `--exact`

Will only run a test with a name exactly matching the test filter.
Test filter must be a whole qualified test name e.g. `src::my_test` instead of just `my_test`.

## `--exit-first`

Stop executing tests after the first failed test.

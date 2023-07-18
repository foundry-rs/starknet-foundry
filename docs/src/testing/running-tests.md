# Running Tests

To run tests with Forge, simply run the `forge` command from the package directory.

```shell
$ snforge
Collected 3 test(s) and 1 test file(s)
Running 1 test(s) from src/lib.cairo
[PASS] src::executing
[PASS] src::calling
[PASS] src::calling_another
Tests: 2 passed, 0 failed, 0 skipped
```

## Filtering Tests

You can pass a filter string after the `forge` command to filter tests.
By default, any test name matching the filter will be run.

```shell
$ snforge calling
Collected 2 test(s) and 1 test file(s)
Running 1 test(s) from src/lib.cairo
[PASS] src::calling
[PASS] src::calling_another
Tests: 2 passed, 0 failed, 0 skipped
```

## Running a Specific Test

To run a specific test, you can pass a filter string along with an `--exact` flag.
Note, you have to use a fully qualified test name, including a module name.

```shell
$ snforge src::calling --exact
Collected 1 test(s) and 1 test file(s)
Running 1 test(s) from src/lib.cairo
[PASS] src::calling
Tests: 1 passed, 0 failed, 0 skipped
```

## Stopping Test Execution After First Failed Test

To stop the test execution after first failed test, you can pass an `--exit-first` flag along with `forge` command.

```shell
$ snforge --exit-first
Collected 5 test(s) and 1 test file(s)
Running 1 test(s) from src/lib.cairo
[PASS] src::executing
[PASS] src::calling
[PASS] src::calling_another
[FAIL] src::failing

Failure data:
    original value: [1234], converted to a string: [failing check]
    
[SKIP] src::other_test
[SKIP] src::yet_another_test
Tests: 3 passed, 1 failed, 2 skipped
```

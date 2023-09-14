# Running Tests

To run tests with Forge, simply run the `snforge` command from the package directory.

```shell
$ snforge
Collected 3 test(s) and 1 test file(s)
Running 3 test(s) from package_name package
[PASS] package_name::executing
[PASS] package_name::calling
[PASS] package_name::calling_another
Tests: 3 passed, 0 failed, 0 skipped
```

## Filtering Tests

You can pass a filter string after the `snforge` command to filter tests.
By default, any test with an [absolute module tree path](https://book.cairo-lang.org/ch06-03-paths-for-referring-to-an-item-in-the-module-tree.html?highlight=path#paths-for-referring-to-an-item-in-the-module-tree)
 matching the filter will be run.

```shell
$ snforge calling
Collected 2 test(s) and 1 test file(s)
Running 2 test(s) from package_name package
[PASS] package_name::calling
[PASS] package_name::calling_another
Tests: 2 passed, 0 failed, 0 skipped
```

## Running a Specific Test

To run a specific test, you can pass a filter string along with an `--exact` flag.
Note, you have to use a fully qualified test name, including a module name.

```shell
$ snforge package_name::calling --exact
Collected 1 test(s) and 1 test file(s)
Running 1 test(s) from package_name package
[PASS] package_name::calling
Tests: 1 passed, 0 failed, 0 skipped
```

## Stopping Test Execution After First Failed Test

To stop the test execution after first failed test, you can pass an `--exit-first` flag along with `snforge` command.

```shell
$ snforge --exit-first
Collected 6 test(s) and 1 test file(s)
Running 6 test(s) from package_name package
[PASS] package_name::executing
[PASS] package_name::calling
[PASS] package_name::calling_another
[FAIL] package_name::failing

Failure data:
    original value: [1234], converted to a string: [failing check]
    
[SKIP] package_name::other_test
[SKIP] package_name::yet_another_test
Tests: 3 passed, 1 failed, 2 skipped
```

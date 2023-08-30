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
By default, any test name matching the filter will be run.

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

## Running tests in Scarb Workspace

When running `snforge` in a Scarb workspace with a root package, it will only run tests inside the root package. When it's ran in a virtual workspace it will execute tests for all packages.  

For a project structure like this

```shell
$ tree . -L 3
.
├── Scarb.toml
├── crates
│   ├── addition
│   │   ├── Scarb.toml
│   │   ├── src
│   │   └── tests
│   └── fibonacci
│       ├── Scarb.toml
│       └── src
└── src
    └── lib.cairo
$ snforge
Collected 1 test(s) and 1 test file(s)
Running 1 test(s) from hello_workspaces package
[PASS] hello_workspaces::test_simple
Tests: 1 passed, 0 failed, 0 skipped
```

To specify a package to test, pass a `--package package_name` (or `-p package_name` for short) flag, to select the specific package. You can also run `snforge` from the package directory for the same effect.

```shell
$ snforge --package addition
Collected 4 test(s) and 3 test file(s)
Running 1 test(s) from addition package
[PASS] addition::tests::it_works
Running 2 test(s) from tests/nested/test_nested.cairo
[PASS] test_nested::test_two
[PASS] test_nested::test_two_and_two
Running 1 test(s) from tests/test_simple.cairo
[PASS] test_simple::simple_case
Tests: 4 passed, 0 failed, 0 skipped
```

You can also pass `--workspace` flag to explicitly run tests for all packages in the workspace.

```shell
$ snforge --workspace
Collected 4 test(s) and 3 test file(s)
Running 1 test(s) from addition package
[PASS] addition::tests::it_works
Running 2 test(s) from tests/nested/test_nested.cairo
[PASS] test_nested::test_two
[PASS] test_nested::test_two_and_two
Running 1 test(s) from tests/test_simple.cairo
[PASS] test_simple::simple_case
Tests: 4 passed, 0 failed, 0 skipped
Running 1 test(s) from fibonacci package
[PASS] fibonacci::tests::it_works
Tests: 1 passed, 0 failed, 0 skipped
Collected 3 test(s) and 2 test file(s)
Running 1 test(s) from hello_workspaces package
[PASS] hello_workspaces::test_simple
Tests: 1 passed, 0 failed, 0 skipped
```

You can read more about Scarb Workspaces in Scarb docs [here](https://docs.swmansion.com/scarb/docs/reference/workspaces.html).

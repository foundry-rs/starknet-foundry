# Running Tests

To run tests with Forge, simply run the `snforge test` command from the package directory.

```shell
$ snforge test
Collected 3 test(s) from package_name package
Running 3 test(s) from src/
[PASS] package_name::tests::executing
[PASS] package_name::tests::calling
[PASS] package_name::tests::calling_another
Tests: 3 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

## Filtering Tests

You can pass a filter string after the `snforge test` command to filter tests.
By default, any test with an [absolute module tree path](https://book.cairo-lang.org/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#paths-for-referring-to-an-item-in-the-module-tree)
 matching the filter will be run.

```shell
$ snforge test calling
Collected 2 test(s) from package_name package
Running 2 test(s) from src/
[PASS] package_name::tests::calling
[PASS] package_name::tests::calling_another
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 1 filtered out
```

## Running a Specific Test

To run a specific test, you can pass a filter string along with an `--exact` flag.
Note, you have to use a fully qualified test name, including a module name.

```shell
$ snforge test package_name::tests::calling --exact
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] package_name::tests::calling
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 2 filtered out
```

## Stopping Test Execution After First Failed Test

To stop the test execution after first failed test, you can pass an `--exit-first` flag along with `snforge test` command.

```shell
$ snforge test --exit-first
Collected 6 test(s) from package_name package
Running 6 test(s) from src/
[PASS] package_name::tests::executing
[PASS] package_name::tests::calling
[PASS] package_name::tests::calling_another
[FAIL] package_name::tests::failing

Failure data:
    original value: [8111420071579136082810415440747], converted to a string: [failing check]

Tests: 3 passed, 1 failed, 2 skipped, 0 ignored, 0 filtered out

Failures:
    package_name::tests::failing
```

## Scarb workspaces support

Forge supports Scarb Workspaces.
To make sure you know how workspaces work,
check Scarb documentation [here](https://docs.swmansion.com/scarb/docs/reference/workspaces.html).

### Workspaces with root package

When running `snforge test` in a Scarb workspace with a root package, it will only run tests inside the root package.  

For a project structure like this

```shell
$ tree . -L 3
.
├── Scarb.toml
├── crates
│   ├── addition
│   │   ├── Scarb.toml
│   │   ├── src
│   │   └── tests
│   └── fibonacci
│       ├── Scarb.toml
│       └── src
├── tests
│   └── test.cairo
└── src
    └── lib.cairo
```

only the tests in `./src` and `./tests` folders will be executed.


```shell

$ snforge test
Collected 1 test(s) from hello_workspaces package
Running 1 test(s) from src/
[PASS] hello_workspaces::tests::test_simple
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

To select the specific package to test, pass a `--package package_name` (or `-p package_name` for short) flag.
You can also run `snforge test` from the package directory to achieve the same effect.

```shell
$ snforge test --package addition
Collected 2 test(s) from addition package
Running 1 test(s) from src/
[PASS] addition::tests::it_works
Running 1 test(s) from tests/
[PASS] tests::test_simple::simple_case
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

You can also pass `--workspace` flag to run tests for all packages in the workspace.

```shell
$ snforge test --workspace
Collected 2 test(s) from addition package
Running 1 test(s) from src/
[PASS] addition::tests::it_works
Running 1 test(s) from tests/
[PASS] tests::test_simple::simple_case
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out


Collected 1 test(s) from fibonacci package
Running 1 test(s) from src/
[PASS] fibonacci::tests::it_works
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out


Collected 1 test(s) from hello_workspaces package
Running 1 test(s) from src/
[PASS] hello_workspaces::tests::test_simple
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

`--package` and `--workspace` flags are mutually exclusive, adding both of them to a `snforge test` command will result in an error.

### Virtual workspaces

Running `snforge test` command in a virtual workspace (a workspace without a root package)
outside any package will by default run tests for all the packages. 
It is equivalent to running `snforge test` with the `--workspace` flag.

To select a specific package to test,
you can use the `--package` flag the same way as in regular workspaces or run `snforge test` from the package directory.

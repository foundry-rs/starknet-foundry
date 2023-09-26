# Running Tests

To run tests with Forge, simply run the `snforge` command from the package directory.

```shell
$ snforge
Collected 3 test(s) from package_name package
Running 3 test(s) from src/
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
Collected 2 test(s) from package_name package
Running 2 test(s) from src/
[PASS] package_name::calling
[PASS] package_name::calling_another
Tests: 2 passed, 0 failed, 0 skipped
```

## Running a Specific Test

To run a specific test, you can pass a filter string along with an `--exact` flag.
Note, you have to use a fully qualified test name, including a module name.

```shell
$ snforge package_name::calling --exact
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] package_name::calling
Tests: 1 passed, 0 failed, 0 skipped
```

## Stopping Test Execution After First Failed Test

To stop the test execution after first failed test, you can pass an `--exit-first` flag along with `snforge` command.

```shell
$ snforge --exit-first
Collected 6 test(s) from package_name package
Running 6 test(s) from src/
[PASS] package_name::executing
[PASS] package_name::calling
[PASS] package_name::calling_another
[FAIL] package_name::failing

Failure data:
    original value: [8111420071579136082810415440747], converted to a string: [failing check]
    
[SKIP] package_name::other_test
[SKIP] package_name::yet_another_test
Tests: 3 passed, 1 failed, 2 skipped

Failures:
    package_name::failing
```

## Scarb workspaces support

Forge supports Scarb Workspaces.
To make sure you know how workspaces work,
check Scarb documentation [here](https://docs.swmansion.com/scarb/docs/reference/workspaces.html).

### Workspaces with root package

When running `snforge` in a Scarb workspace with a root package, it will only run tests inside the root package.  

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

$ snforge
Collected 1 test(s) from hello_workspaces package
Running 1 test(s) from src/
[PASS] hello_workspaces::test_simple
Tests: 1 passed, 0 failed, 0 skipped
```

To select the specific package to test, pass a `--package package_name` (or `-p package_name` for short) flag.
You can also run `snforge` from the package directory to achieve the same effect.

```shell
$ snforge --package addition
Collected 2 test(s) from addition package
Running 1 test(s) from src/
[PASS] addition::tests::it_works
Running 1 test(s) from tests/
[PASS] tests::test_simple::simple_case
Tests: 2 passed, 0 failed, 0 skipped
```

You can also pass `--workspace` flag to run tests for all packages in the workspace.

```shell
$ snforge --workspace
Collected 2 test(s) from addition package
Running 1 test(s) from src/
[PASS] addition::tests::it_works
Running 1 test(s) from tests/
[PASS] tests::test_simple::simple_case
Tests: 2 passed, 0 failed, 0 skipped


Collected 1 test(s) from fibonacci package
Running 1 test(s) from src/
[PASS] fibonacci::tests::it_works
Tests: 1 passed, 0 failed, 0 skipped


Collected 1 test(s) from hello_workspaces package
Running 1 test(s) from src/
[PASS] hello_workspaces::test_simple
Tests: 1 passed, 0 failed, 0 skipped
```

`--package` and `--workspace` flags are mutually exclusive, adding both of them to a `snforge` command will result in an error.

### Virtual workspaces

Running `snforge` command in a virtual workspace (a workspace without a root package)
outside any package will by default run tests for all the packages. 
It is equivalent to running `snforge` with the `--workspace` flag.

To select a specific package to test,
you can use the `--package` flag the same way as in regular workspaces or run `snforge` from the package directory.

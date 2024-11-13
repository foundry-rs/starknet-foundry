# Testing Scarb Workspaces

`snforge` supports Scarb Workspaces.
To make sure you know how workspaces work,
check Scarb documentation [here](https://docs.swmansion.com/scarb/docs/reference/workspaces.html).

## Workspaces With Root Package

When running `snforge test` in a Scarb workspace with a root package, it will only run tests inside the root package.  

For a project structure like this

```shell
$ tree . -L 3
```

<details>
<summary>Output:</summary>

```shell
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
</details>
<br>

only the tests in `./src` and `./tests` folders will be executed.

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from hello_workspaces package
Running 1 test(s) from src/
[PASS] hello_workspaces::tests::test_simple
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```
</details>
<br>

To select the specific package to test, pass a `--package package_name` (or `-p package_name` for short) flag.
You can also run `snforge test` from the package directory to achieve the same effect.

<!-- package_name=hello_workspaces -->
```shell
$ snforge test --package addition
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from addition package
Running 1 test(s) from src/
[PASS] addition::tests::it_works
Running 1 test(s) from tests/
[PASS] tests::test_simple::simple_case
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```
</details>
<br>

You can also pass `--workspace` flag to run tests for all packages in the workspace.

```shell
$ snforge test --workspace
```

<details>
<summary>Output:</summary>

```shell
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
</details>
<br>

`--package` and `--workspace` flags are mutually exclusive, adding both of them to a `snforge test` command will result in an error.

## Virtual Workspaces

Running `snforge test` command in a virtual workspace (a workspace without a root package)
outside any package will by default run tests for all the packages. 
It is equivalent to running `snforge test` with the `--workspace` flag.

To select a specific package to test,
you can use the `--package` flag the same way as in regular workspaces or run `snforge test` from the package directory.

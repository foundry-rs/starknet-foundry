# First Steps With Starknet Foundry

In this section we provide an overview of Starknet Foundry `snforge` command line tool.
We demonstrate how to create a new project, compile, and test it.

To start a new project with Starknet Foundry, run `snforge init`

```shell
$ snforge init project_name
```

Let's check out the project structure

```shell
$ cd project_name
$ tree . -L 1
.
├── README.md
├── Scarb.toml
├── src
└── tests

3 directories
```

* `src/` contains source code of all your contracts.
* `tests/` contains tests.
* `Scarb.toml` contains configuration of the project as well as of `snforge`, `sncast` etc.

And run tests with `snforge test`

```shell
$ snforge test
Collected 2 test(s) from test_name package
Running 0 test(s) from src/
Running 2 test(s) from tests/
[PASS] tests::test_contract::test_increase_balance
[PASS] tests::test_contract::test_cannot_increase_balance_with_zero_value
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

## Using `snforge` With Existing Scarb Projects

To use `snforge` with existing Scarb projects, make sure you have declared the `snforge_std` package as your project
dependency.

Add the following line under `[dependencies]` section in the `Scarb.toml` file.

```toml
# ...

[dependencies]
snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
```

Make sure that the version in `tag` matches `snforge`. You can check the currently installed version with

```shell
$ snforge --version
snforge 0.12.0
```

It is also possible to add this dependency
using [`scarb add`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency-via-scarb-add)
command.

```shell
$ scarb add snforge_std \
 --git https://github.com/foundry-rs/starknet-foundry.git \
 --tag v0.12.0
```

Additionally, ensure that `casm` codegen is enabled in the `Scarb.toml` file.

```toml
# ...
[[target.starknet-contract]]
casm = true
# ...
```

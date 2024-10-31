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
├── Scarb.lock
├── Scarb.toml
├── src
└── tests

2 directories, 2 files
```

* `src/` contains source code of all your contracts.
* `tests/` contains tests.
* `Scarb.toml` contains configuration of the project as well as of `snforge`
* `Scarb.lock` a locking mechanism to achieve reproducible dependencies when installing the project locally  

And run tests with `snforge test`

```shell
$ snforge test
   Compiling project_name v0.1.0 (project_name/Scarb.toml)
    Finished release target(s) in 1 second

Collected 2 test(s) from project_name package
Running 0 test(s) from src/
Running 2 test(s) from tests/
[PASS] tests::test_contract::test_increase_balance (gas: ~170)
[PASS] tests::test_contract::test_cannot_increase_balance_with_zero_value (gas: ~104)
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored
```

## Using `snforge` With Existing Scarb Projects

To use `snforge` with existing Scarb projects, make sure you have declared the `snforge_std` package as your project
development dependency.

Add the following line under `[dev-dependencies]` section in the `Scarb.toml` file.

```toml
# ...

[dev-dependencies]
snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.27.0" }
```

Make sure that the version in `tag` matches `snforge`. You can check the currently installed version with

```shell
$ snforge --version
snforge 0.27.0
```

It is also possible to add this dependency
using [`scarb add`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency-via-scarb-add)
command.

```shell
$ scarb add snforge_std \ 
 --dev \
 --git https://github.com/foundry-rs/starknet-foundry.git \
 --tag v0.27.0
```

Additionally, ensure that starknet-contract target is enabled in the `Scarb.toml` file.

```toml
# ...
[[target.starknet-contract]]

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
```

<details>
<summary>Output:</summary>

```shell
.
├── Scarb.lock
├── Scarb.toml
├── src
└── tests

2 directories, 2 files
```
</details>
<br>

* `src/` contains source code of all your contracts.
* `tests/` contains tests.
* `Scarb.toml` contains configuration of the project as well as of `snforge`
* `Scarb.lock` a locking mechanism to achieve reproducible dependencies when installing the project locally  

And run tests with `snforge test`

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
   Compiling project_name v0.1.0 (project_name/Scarb.toml)
    Finished release target(s) in 1 second

Collected 2 test(s) from project_name package
Running 0 test(s) from src/
Running 2 test(s) from tests/
[PASS] tests::test_contract::test_increase_balance (gas: ~170)
[PASS] tests::test_contract::test_cannot_increase_balance_with_zero_value (gas: ~104)
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored
```
</details>
<br>

## Using `snforge` With Existing Scarb Projects

To use `snforge` with existing Scarb projects, make sure you have declared the `snforge_std` package as your project
development dependency.

Add the following line under `[dev-dependencies]` section in the `Scarb.toml` file.

```toml
# ...

[dev-dependencies]
snforge_std = "0.33.0"
```

Make sure that the above version matches the installed `snforge` version. You can check the currently installed version with

```shell
$ snforge --version
```

<details>
<summary>Output:</summary>

```shell
snforge 0.33.0
```
</details>
<br>

It is also possible to add this dependency
using [`scarb add`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency-via-scarb-add)
command.

```shell
$ scarb add snforge_std@0.33.0 --dev
```

Additionally, ensure that starknet-contract target is enabled in the `Scarb.toml` file.

```toml
# ...
[[target.starknet-contract]]
```
# First Steps With Starknet Foundry

In this section we provide an overview of Starknet Foundry `snforge` command line tool.
We demonstrate how to create a new project, compile, and test it.

To start a new project with Starknet Foundry, run `snforge init`

```shell
$ snforge init hello_starknet
```

Let's check out the project structure

```shell
$ cd hello_starknet
$ tree . -L 1
```

<details>
<summary>Output:</summary>

```shell
.
├── Scarb.lock
├── Scarb.toml
├── snfoundry.toml
├── src
└── tests

2 directories, 3 files
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
Collected 2 test(s) from hello_starknet package
Running 0 test(s) from src/
Running 2 test(s) from tests/
[PASS] hello_starknet_integrationtest::test_contract::test_cannot_increase_balance_with_zero_value (gas: ~105)
[PASS] hello_starknet_integrationtest::test_contract::test_increase_balance (gas: ~172)
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
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

<!-- ignore -->
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
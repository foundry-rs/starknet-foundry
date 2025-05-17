# First Steps With Starknet Foundry

In this section we provide an overview of Starknet Foundry `snforge` command line tool.
We demonstrate how to create a new project, compile, and test it.

To start a new project with Starknet Foundry, run `snforge new`

```shell
$ snforge new hello_starknet
```

> 📝 **Note**
>
> By default, `snforge new` creates a project with a simple `HelloStarknet` contract. You can create a different project using the `--template` flag. 
> To see the list of available templates, refer to the [snforge new documentation](../appendix/snforge/new.md#-t---template)

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
[PASS] hello_starknet_integrationtest::test_contract::test_cannot_increase_balance_with_zero_value (l1_gas: ~0, l1_data_gas: ~96, l2_gas: ~360000)
[PASS] hello_starknet_integrationtest::test_contract::test_increase_balance (l1_gas: ~0, l1_data_gas: ~192, l2_gas: ~480000)
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
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

<!-- { "ignored": true } -->
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

> 📝 **Note**
>
> You can additionally specify `scarb` settings to avoid compiling Cairo plugin which `snforge_std` depends on. The plugin is written in Rust and, by default, is compiled locally on the user's side.
> ```
> [tool.scarb]  
> allow-prebuilt-plugins = ["snforge_std"]
> ```
> This configuration requires Scarb version >= 2.10.0 .
>

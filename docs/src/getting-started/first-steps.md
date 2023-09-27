# First Steps With Starknet Foundry

In this section we provide an overview of Starknet Foundry `forge` command line tool.
We demonstrate how to create a new project, compile, and test it.

To start a new project with Starknet Foundry, run `--init` command:

```shell
snforge --init project_name
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

Ensures that `casm` codegen is enabled in the `Scarb.toml` file.
```toml
# ...
[[target.starknet-contract]]
casm = true
# ...
```

And run tests with `snforge`

```shell
$ snforge
Collected 2 test(s) from test_name package
Running 0 test(s) from src/
Running 2 test(s) from tests/
[PASS] tests::test_contract::test_increase_balance
[PASS] tests::test_contract::test_cannot_increase_balance_with_zero_value
Tests: 2 passed, 0 failed, 0 skipped
```

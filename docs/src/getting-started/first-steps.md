# First Steps With Starknet-Foundry

In this section we provide an overview of Starknet-Foundry `forge` command line tool. We demonstrate how to create a
new project, compile, and test it.

To start a new project with Starknet-Foundry, clone the template repository

```shell
$ git clone https://github.com/foundry-rs/starknet_forge_template.git
```

Let's check out the project structure

```shell
$ cd starknet_forge_template
$ tree . -L 1
.
├── README.md
├── Scarb.toml
├── src
└── tests

3 directories
```

* `src/` contains source code of all your contracts.
* `tests/` contains tests. Note that test can also be included in any file or directory.
* `Scarb.toml` contains configuration of the project as well as of `forge`, `cast` etc.

And run tests with `forge`

```shell
$ forge
Collected 2 test(s) and 2 test file(s)
Running 0 test(s) from src/lib.cairo
Running 2 test(s) from tests/test_contract.cairo
[PASS] test_contract::test_contract::test_increase_balance
[PASS] test_contract::test_contract::test_cannot_increase_balance_with_zero_value
Tests: 2 passed, 0 failed, 0 skipped
```

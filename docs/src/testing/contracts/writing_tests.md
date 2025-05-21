# Writing Tests

## The Test Contract

Let's consider a simple smart contract with two methods.

```rust
{{#include ../../../listings/testing_smart_contracts_writing_tests/src/lib.cairo}}
```

Note that the name after `mod` will be used as the contract name for testing purposes.

## Writing Tests

Let's write a test that will deploy the `SimpleContract` contract and call some functions.

```rust
{{#include ../../../listings/testing_smart_contracts_writing_tests/tests/simple_contract.cairo}}
```

> 📝 **Note**
>
> Notice that the arguments to the contract's constructor (the `deploy`'s `calldata` argument) need to be serialized with `Serde`.
>
> `SimpleContract` contract has no constructor, so the calldata remains empty in the example above.

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from testing_smart_contracts_handling_errors package
Running 2 test(s) from tests/
[FAIL] testing_smart_contracts_handling_errors_integrationtest::panic::failing

Failure data:
    (0x50414e4943 ('PANIC'), 0x444159544148 ('DAYTAH'))

[PASS] testing_smart_contracts_handling_errors_integrationtest::handle_panic::handling_string_errors (l1_gas: ~0, l1_data_gas: ~96, l2_gas: ~280000)
Running 0 test(s) from src/
Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

Failures:
    testing_smart_contracts_handling_errors_integrationtest::panic::failing
```
</details>
<br>
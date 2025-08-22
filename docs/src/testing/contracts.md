# Testing Smart Contracts

> ℹ️ **Info**
>
> To use the library functions designed for testing smart contracts,
> you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using the appropriate version.
>```toml
> [dev-dependencies]
> snforge_std = "{{snforge_std_version}}"
> ```

Using unit testing as much as possible is a good practice, as it makes your test suites run faster. However, when
writing smart contracts, you often want to test their interactions with the blockchain state and with other contracts.

## The Test Contract

Let's consider a simple smart contract with two methods.

```rust
{{#include ../../listings/testing_smart_contracts_writing_tests/src/lib.cairo}}
```

Note that the name after `mod` will be used as the contract name for testing purposes.

## Writing Tests

Let's write a test that will deploy the `SimpleContract` contract and call some functions.

```rust
{{#include ../../listings/testing_smart_contracts_writing_tests/tests/simple_contract.cairo}}
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
Tests: 1 passed, 1 failed, 0 ignored, 0 filtered out

Failures:
    testing_smart_contracts_handling_errors_integrationtest::panic::failing
```
</details>
<br>

## Handling Errors

Sometimes we want to test contracts functions that can panic, like testing that function that verifies caller address
panics on invalid address. For that purpose Starknet also provides a `SafeDispatcher`, that returns a `Result` instead of
panicking.

First, let's add a new, panicking function to our contract.

```rust
{{#include ../../listings/testing_smart_contracts_handling_errors/src/lib.cairo}}
```

If we called this function in a test, it would result in a failure.

```rust
{{#include ../../listings/testing_smart_contracts_handling_errors/tests/panic.cairo:first_half}}
{{#include ../../listings/testing_smart_contracts_handling_errors/tests/panic.cairo:second_half}}
```

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
Tests: 1 passed, 1 failed, 0 ignored, 0 filtered out

Failures:
    testing_smart_contracts_handling_errors_integrationtest::panic::failing
```
</details>
<br>

### `SafeDispatcher`

Using `SafeDispatcher` we can test that the function in fact panics with an expected message.
Safe dispatcher is a special kind of dispatcher that allows using the contract without automatically unwrapping the result, thereby making possible to catch the error like shown below.

```rust
{{#include ../../listings/testing_smart_contracts_safe_dispatcher/tests/safe_dispatcher.cairo}}
```

Now the test passes as expected.

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from testing_smart_contracts_safe_dispatcher package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] testing_smart_contracts_safe_dispatcher_integrationtest::safe_dispatcher::handling_errors (l1_gas: ~0, l1_data_gas: ~96, l2_gas: ~280000)
Tests: 1 passed, 0 failed, 0 ignored, 0 filtered out
```
</details>
<br>

> 📝 **Note**
>
> It is not possible to catch errors that cause immediate termination of execution, e.g. calling a contract with a nonexistent address.
> A full list of such errors can be found [here](https://community.starknet.io/t/starknet-v0-13-4-pre-release-notes/115257#p-2358763-catching-errors-12).

Similarly, you can handle the panics which use `ByteArray` as an argument (like an `assert!` or `panic!` macro)

```rust
{{#include ../../listings/testing_smart_contracts_handling_errors/tests/handle_panic.cairo}}
```
You also could skip the de-serialization of the `panic_data`, and not use `try_deserialize_bytearray_error`, but this way you can actually use assertions on the `ByteArray` that was used to panic.

> 📝 **Note**
>
> To operate with `SafeDispatcher` it's required to annotate its usage with `#[feature("safe_dispatcher")]`.
>
> There are 3 options:
> - module-level declaration
>   ```rust
>   #[feature("safe_dispatcher")]
>   mod my_module;
>   ```
> - function-level declaration
>   ```rust
>   #[feature("safe_dispatcher")]
>   fn my_function() { ... }
>   ```
> - directly before the usage
>   ```rust
>   #[feature("safe_dispatcher")]
>   let result = safe_dispatcher.some_function();
>   ```

### Expecting Test Failure

Sometimes the test code failing can be a desired behavior.
Instead of manually handling it, you can simply mark your test as `#[should_panic(...)]`.
[See here](./testing.md#expected-failures) for more details.

## Passing Constructor Arguments

The previous example was a basic one. However, sometimes you may need to pass arguments to contract's constructor. This can be done in two ways:
- With manual serialization
- With `deploy_for_test` function (available since Cairo 2.12)

Let's compare both approaches.

### Test Contract

Below contract simulates a basic shopping cart. Its constructor takes initial products which are vector of `Product` structs.

```rust
{{#include ../../listings/deployment_with_constructor_args/src/lib.cairo}}
```

### Deployment with `deploy_for_test`

`deploy_for_test` is an utility function that simplifies the deployment process by automatically handling serialization of constructor parameters.

```rust
{{#include ../../listings/deployment_with_constructor_args/tests/test_with_deploy_for_test.cairo}}
```

### Deployment with Manual Serialization

In this case we need to manually serialize the constructor parameters and pass them as calldata to the `deploy` function.

```rust
{{#include ../../listings/deployment_with_constructor_args/tests/test_with_serialization.cairo}}
```

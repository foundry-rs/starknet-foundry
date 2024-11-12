# Testing Smart Contracts

> ℹ️ **Info**
>
> To use the library functions designed for testing smart contracts,
> you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using the appropriate version.
>```toml
> [dev-dependencies]
> snforge_std = "0.33.0"
> ```

Using unit testing as much as possible is a good practice, as it makes your test suites run faster. However, when
writing smart contracts, you often want to test their interactions with the blockchain state and with other contracts.

## The Test Contract

Let's consider a simple smart contract with two methods.

```rust
{{#include ../../listings/snforge_overview/crates/testing_smart_contracts/src/simple_contract.cairo}}
```

Note that the name after `mod` will be used as the contract name for testing purposes.

## Writing Tests

Let's write a test that will deploy the `HelloStarknet` contract and call some functions.

```rust
{{#include ../../listings/snforge_overview/crates/testing_smart_contracts/tests/simple_contract.cairo}}
```

> 📝 **Note**
>
> Notice that the arguments to the contract's constructor (the `deploy`'s `calldata` argument) need to be serialized with `Serde`.
>
> `HelloStarknet` contract has no constructor, so the calldata remains empty in the example above.

```shell
$ snforge test
Collected 1 test(s) from testing_smart_contracts package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] tests::call_and_invoke
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

## Handling Errors

Sometimes we want to test contracts functions that can panic, like testing that function that verifies caller address
panics on invalid address. For that purpose Starknet also provides a `SafeDispatcher`, that returns a `Result` instead of
panicking.

First, let's add a new, panicking function to our contract.

```rust
{{#include ../../listings/snforge_overview/crates/testing_smart_contracts/src/handling_errors.cairo}}
```

If we called this function in a test, it would result in a failure.

```rust
{{#include ../../listings/snforge_overview/crates/testing_smart_contracts/tests/panic.cairo:first_half}}
{{#include ../../listings/snforge_overview/crates/testing_smart_contracts/tests/panic.cairo:second_half}}
```

```shell
$ snforge test
Collected 1 test(s) from testing_smart_contracts package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[FAIL] tests::failing

Failure data:
    (0x50414e4943 ('PANIC'), 0x444159544148 ('DAYTAH'))

Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

Failures:
    tests::failing
```

### `SafeDispatcher`

Using `SafeDispatcher` we can test that the function in fact panics with an expected message.
Safe dispatcher is a special kind of dispatcher, which are not allowed in contracts themselves,
but are available for testing purposes.

They allow using the contract without automatically unwrapping the result, which allows to catch the error like shown below.

```rust
{{#include ../../listings/snforge_overview/crates/testing_smart_contracts/tests/safe_dispatcher.cairo}}
```

Now the test passes as expected.

```shell
$ snforge test
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] tests::handling_errors
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

Similarly, you can handle the panics which use `ByteArray` as an argument (like an `assert!` or `panic!` macro)

```rust
{{#include ../../listings/snforge_overview/crates/testing_smart_contracts/tests/handle_panic.cairo}}
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

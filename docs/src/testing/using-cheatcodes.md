# Using Cheatcodes

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using the appropriate version.
>
> ```toml
> [dev-dependencies]
> snforge_std = "{{snforge_std_version}}"
> ```

When testing smart contracts, often there are parts of code that are dependent on a specific blockchain state.
Instead of trying to replicate these conditions in tests, you can emulate them
using [cheatcodes](../appendix/cheatcodes.md).

> ⚠️ **Warning**
> 
> These examples make use of `assert_macros`, so it's recommended to get familiar with them first. [Learn more about `assert_macros`](testing.md#writing-assertions-and-assert_macros-package)

## The Test Contract

In this tutorial, we will be using the following Starknet contract:

```rust
{{#include ../../listings/using_cheatcodes/src/lib.cairo}}
```

## Writing Tests

We can try to create a test that will increase and verify the balance.

```rust
{{#include ../../listings/using_cheatcodes/tests/lib.cairo}}
```

This test fails, which means that `increase_balance` method panics as we expected.

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from using_cheatcodes package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[FAIL] using_cheatcodes_tests::call_and_invoke

Failure data:
    0x75736572206973206e6f7420616c6c6f776564 ('user is not allowed')

Tests: 0 passed, 1 failed, 0 ignored, 0 filtered out

Failures:
    using_cheatcodes_tests::call_and_invoke
```
</details>
<br>

Our user validation is not letting us call the contract, because the default caller address is not `123`.

## Using Cheatcodes in Tests

By using cheatcodes, we can change various properties of transaction info, block info, etc.
For example, we can use the [`start_cheat_caller_address`](../appendix/cheatcodes/caller_address.md) cheatcode to change the caller
address, so it passes our validation.

### Cheating an Address

```rust
{{#include ../../listings/using_cheatcodes_cheat_address/tests/lib.cairo}}
```

The test will now pass without an error

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from using_cheatcodes_cheat_address package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] using_cheatcodes_cheat_address_tests::call_and_invoke (l1_gas: ~0, l1_data_gas: ~288, l2_gas: ~600000)
Tests: 1 passed, 0 failed, 0 ignored, 0 filtered out
```
</details>
<br>

### Cancelling the Cheat

Most cheatcodes come with corresponding `start_` and `stop_` functions that can be used to start and stop the state
change.
In case of the `start_cheat_caller_address`, we can cancel the address change
using [`stop_cheat_caller_address`](../appendix/cheatcodes/caller_address.md#stop_cheat_caller_address).
We will demonstrate its behavior using `SafeDispatcher` to show when exactly the fail occurs:

```rust
{{#include ../../listings/using_cheatcodes_cancelling_cheat/tests/lib.cairo}}
```

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from using_cheatcodes_cancelling_cheat package
Running 1 test(s) from tests/
[FAIL] using_cheatcodes_cancelling_cheat_tests::call_and_invoke

Failure data:
    0x5365636f6e642063616c6c206661696c656421 ('Second call failed!')

Running 0 test(s) from src/
Tests: 0 passed, 1 failed, 0 ignored, 0 filtered out

Failures:
    using_cheatcodes_cancelling_cheat_tests::call_and_invoke
```
</details>
<br>

We see that the second `increase_balance` fails since we cancelled the cheatcode.

### Cheating Addresses Globally

In case you want to cheat the caller address for all contracts, you can use the global cheatcode which has the `_global` suffix. Note, that we don't specify target, nor the span, because this cheatcode type works globally and indefinitely.
For more see [Cheating Globally](../appendix/cheatcodes/global.md).

```rust
{{#include ../../listings/using_cheatcodes_others/tests/caller_address/proper_use_global.cairo}}
```

### Cheating the Constructor

Most of the cheatcodes like `cheat_caller_address`, `mock_call`, `cheat_block_timestamp`, `cheat_block_number`, `elect` do work in the constructor of the contracts.

Let's say, that you have a contract that saves the caller address (deployer) in the constructor, and you want it to be pre-set to a certain value.

To `cheat_caller_address` the constructor, you need to `start_cheat_caller_address` before it is invoked, with the right address. To achieve this, you need to precalculate the address of the contract by using the `precalculate_address` function of `ContractClassTrait` on the declared contract, and then use it in `start_cheat_caller_address` as an argument:

```rust
{{#include ../../listings/using_cheatcodes_others/tests/cheat_constructor.cairo}}
```

### Setting Cheatcode Span

Sometimes it's useful to have a cheatcode work only for a certain number of target calls.

That's where [`CheatSpan`](../appendix/cheatcodes/cheat_span.md) comes in handy.

```rust
enum CheatSpan {
    Indefinite: (),
    TargetCalls: NonZero<usize>,
}
```

To set span for a cheatcode, use `cheat_caller_address` / `cheat_block_timestamp` / `cheat_block_number` / etc.

```rust
cheat_caller_address(contract_address, new_caller_address, CheatSpan::TargetCalls(1))
```

Calling a cheatcode with `CheatSpan::TargetCalls(N)` is going to activate the cheatcode for `N` calls to a specified contract address, after which it's going to be automatically canceled.

Of course the cheatcode can still be canceled before its `CheatSpan` goes down to 0 - simply call `stop_cheat_caller_address` on the target manually.

> ℹ️ **Info**
>
> Using `start_cheat_caller_address` is **equivalent** to using `cheat_caller_address` with `CheatSpan::Indefinite`.


To better understand the functionality of `CheatSpan`, here's a full example:

```rust
{{#include ../../listings/using_cheatcodes_others/tests/caller_address/span.cairo}}
```

### Cheating ERC-20 Token balance

If you want to cheat the balance of an ERC-20 token (STRK, ETH or custom one), you can use the [`set_balance`](../appendix/cheatcodes/set_balance.md) cheatcode.

> ℹ️ **Info**
>
> STRK and ETH are predeployed in every test case by default, so you can use them without any additional setup.
> The predeployment can be disabled by the `#[disable_predeployed_contracts]` attribute.

Below is a basic example of setting and reading STRK balance:

```rust
{{#include ../../listings/using_cheatcodes_others/tests/set_balance_strk.cairo}}
```

You can also use `CustomToken` (see [`Token`](../appendix/cheatcodes/token.md) docs). It needs `contract_address` and `balances_variable_selector` (which refers to storage variable, holding the mapping of balances -> amounts):

```rust
{{#include ../../listings/using_cheatcodes_others/tests/set_balance_custom_token.cairo}}
```
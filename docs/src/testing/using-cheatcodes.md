# Using Cheatcodes

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using appropriate release tag.
>
> ```toml
> [dev-dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.9.0" }
> ```

When testing smart contracts, often there are parts of code that are dependent on a specific blockchain state.
Instead of trying to replicate these conditions in tests, you can emulate them
using [cheatcodes](../appendix/cheatcodes.md).

## The Test Contract

In this tutorial, we will be using the following Starknet contract:

```rust
#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn get_block_number_at_construction(self: @TContractState) -> u64;
    fn get_block_timestamp_at_construction(self: @TContractState) -> u64;
}

#[starknet::contract]
mod HelloStarknet {

    use box::BoxTrait;
    use starknet::{Into, get_caller_address};

    #[storage]
    struct Storage {
        balance: felt252,
        blk_nb: u64,
        blk_timestamp: u64,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        // store the current block number
        self.blk_nb.write(starknet::get_block_info().unbox().block_number);
        // store the current block timestamp
        self.blk_timestamp.write(starknet::get_block_info().unbox().block_timestamp);
    }

    #[abi(embed_v0)]
    impl IHelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // Increases the balance by the given amount.
        fn increase_balance(ref self: ContractState, amount: felt252) {
            assert_is_allowed_user();
            self.balance.write(self.balance.read() + amount);
        }
        // Gets the balance.
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
        // Gets the block number
        fn get_block_number_at_construction(self: @ContractState) -> u64 {
            self.blk_nb.read()
        }
        // Gets the block timestamp
        fn get_block_timestamp_at_construction(self: @ContractState) -> u64 {
            self.blk_timestamp.read()
        }
    }

    fn assert_is_allowed_user() {
        // checks if caller is '123'
        let address = get_caller_address();
        assert(address.into() == 123, 'user is not allowed');
    }
}
```

Please note that this contract example is a continuation of the same contract as in the [Testing Smart Contracts](./testing.md) page.

## Writing Tests

We can try to create a test that will increase and verify the balance.

```rust
#[test]
fn call_and_invoke() {
    // ...

    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
```

However, when running this test, we will get a failure with a message

```shell
$ snforge test
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[FAIL] tests::call_and_invoke

Failure data:
    0x75736572206973206e6f7420616c6c6f776564 ('user is not allowed')

Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

Failures:
    tests::call_and_invoke
```

Our user validation is not letting us call the contract, because the default caller address is not `123`.

## Using Cheatcodes in Tests

By using cheatcodes, we can change various properties of transaction info, block info, etc.
For example, we can use the [`start_cheat_caller_address`](../appendix/cheatcodes/caller_address.md) cheatcode to change the caller
address, so it passes our validation.

### Cheating an Address

```rust
use snforge_std::{ declare, ContractClassTrait, start_cheat_caller_address };

#[test]
fn call_and_invoke() {
    let contract = declare("HelloStarknet").unwrap();
    let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    // Change the caller address to 123 when calling the contract at the `contract_address` address
    start_cheat_caller_address(contract_address, 123.try_into().unwrap());

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
```

The test will now pass without an error

```shell
$ snforge test
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] tests::call_and_invoke
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

### Canceling the Cheat

Most cheatcodes come with corresponding `start_` and `stop_` functions that can be used to start and stop the state
change.
In case of the `start_cheat_caller_address`, we can cancel the address change
using [`stop_cheat_caller_address`](../appendix/cheatcodes/caller_address.md#stop_cheat_caller_address)

```rust
use snforge_std::{stop_cheat_caller_address};

#[test]
fn call_and_invoke() {
    // ...

    // The address when calling contract at the `contract_address` address will no longer be changed
    stop_cheat_caller_address(contract_address);

    // This will fail
    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
```

```shell
$ snforge test
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[FAIL] tests::call_and_invoke, 0 ignored, 0 filtered out

Failure data:
    0x75736572206973206e6f7420616c6c6f776564 ('user is not allowed')

Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

Failures:
    tests::call_and_invoke
```

### Cheating the Constructor

Most of the cheatcodes like `cheat_caller_address`, `mock_call`, `cheat_block_timestamp`, `cheat_block_number`, `elect` do work in the constructor of the contracts.

Let's say, that you have a contract that saves the caller address (deployer) in the constructor, and you want it to be pre-set to a certain value.

To `cheat_caller_address` the constructor, you need to `start_cheat_caller_address` before it is invoked, with the right address. To achieve this, you need to precalculate the address of the contract by using the `precalculate_address` function of `ContractClassTrait` on the declared contract, and then use it in `start_cheat_caller_address` as an argument:

```rust
use snforge_std::{ declare, ContractClassTrait, start_cheat_caller_address };

#[test]
fn mock_constructor_with_cheat_caller_address() {
    let contract = declare("HelloStarknet").unwrap();
    let constructor_arguments = @ArrayTrait::new();

    // Precalculate the address to obtain the contract address before the constructor call (deploy) itself
    let contract_address = contract.precalculate_address(constructor_arguments);

    // Change the caller address to 123 before the call to contract.deploy
    start_cheat_caller_address(contract_address, 123.try_into().unwrap());

    // The constructor will have 123 set as the caller address
    contract.deploy(constructor_arguments).unwrap();
}
```

### Setting Cheatcode Span

Sometimes it's useful to have a cheatcode work only for a certain number of target calls. 

That's where [`CheatSpan`](../appendix/cheatcodes/cheat_span.md) comes in handy.

```rust
enum CheatSpan {
    Indefinite: (),
    TargetCalls: usize,
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
use snforge_std::{
    declare, ContractClass, ContractClassTrait, cheat_caller_address, CheatSpan
};

#[test]
#[feature("safe_dispatcher")]
fn call_and_invoke() {
    let contract = declare("HelloStarknet").unwrap();
    let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance = safe_dispatcher.get_balance().unwrap();
    assert_eq!(balance, 0);

    // Function `increase_balance` from HelloStarknet contract
    // requires the caller_address to be 123
    let cheat_caller_addressed_address: ContractAddress = 123.try_into().unwrap();

    // Change the caller address for the contract_address for a span of 2 target calls (here, calls to contract_address)
    cheat_caller_address(contract_address, cheat_caller_addressed_address, CheatSpan::TargetCalls(2));

    // Call #1 should succeed
    let call_1_result = safe_dispatcher.increase_balance(100);
    assert!(call_1_result.is_ok());

    // Call #2 should succeed
    let call_2_result = safe_dispatcher.increase_balance(100);
    assert!(call_2_result.is_ok());

    // Call #3 should fail, as the cheat_caller_address cheatcode has been canceled
    let call_3_result = safe_dispatcher.increase_balance(100);
    assert!(call_3_result.is_err());

    let balance = safe_dispatcher.get_balance().unwrap();
    assert_eq!(balance, 200);
}
```

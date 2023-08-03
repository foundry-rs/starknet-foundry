# Using Cheatcodes

When testing smart contracts, often there are parts of code that are dependent on specific blockchain state or are
triggered on specific transaction properties.
Instead of trying to replicate these conditions in tests, you can emulate them
using [cheatcodes](../appendix/cheatcodes.md).

## The Test Contract

In this tutorial will be using this Starknet contract:

```rust
#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[external(v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // Increases the balance by the given amount.
        fn increase_balance(ref self: ContractState, amount: felt252) {
            assert_is_allowed_user();
            self.balance.write(self.balance.read() + amount);
        }

        // Gets the balance. 
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
    }
    
    fn assert_is_allowed_user() {
        let address = get_caller_address();
        assert(address.into() == 123, 'user is not allowed');
    }
}
```

Note that this is the same contract as on the [Testing Smart Contracts](./testing.md) page with
the `assert_is_allowed_user` function added.

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
$ snforge
Collected 1 test(s) and 1 test file(s)
Running 1 test(s) from src/lib.cairo
[FAIL] src::call_and_invoke

Failure data:
    original value: [1234], converted to a string: [user is not allowed]

Tests: 0 passed, 1 failed, 0 skipped
```

Our user validation is not letting us call the contract, because the default caller address is not `123`.

## Using Cheatcodes in Tests

By using cheatcodes, we can change various properties of transaction info, block info, etc.
For example, we can use the [`start_prank`](../appendix/cheatcodes/start_prank.md) cheatcode, to change the caller
address,
so it passes our validation.

### Pranking the Address

```rust
#[test]
fn call_and_invoke() {
    let class_hash = declare('HelloStarknet');
    let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
    let contract_address = deploy(prepared).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };
    
    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    // Change the caller address to 123 when calling the contract at the `contract_address` address
    start_prank(contract_address, 123.try_into().unwrap());
    
    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
```

The test will now pass without an error

```shell
$ snforge
Collected 1 test(s) and 1 test file(s)
Running 1 test(s) from src/lib.cairo
[PASS] src::call_and_invoke
Tests: 1 passed, 0 failed, 0 skipped
```

### Canceling the Prank

Most cheatcodes come with corresponding `start_` and `stop_` functions that can be used to start and stop the state
change.
In case of the `start_prank`, we can cancel the address change
using [`stop_prank`](../appendix/cheatcodes/stop_prank.md)

```rust
#[test]
fn call_and_invoke() {
    // ...
    
    // The address when calling contract at the `contract_address` address will no longer be changed
    stop_prank(contract_address);
    
    // This will fail
    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
```

```shell
$ snforge
Collected 1 test(s) and 1 test file(s)
Running 1 test(s) from src/lib.cairo
[FAIL] src::call_and_invoke

Failure data:
    original value: [1234], converted to a string: [user is not allowed]

Tests: 0 passed, 1 failed, 0 skipped
```
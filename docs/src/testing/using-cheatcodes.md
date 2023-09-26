# Using Cheatcodes

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency) 
> using appropriate release tag.
>```toml
> [dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.5.0" }
> ```

When testing smart contracts, often there are parts of code that are dependent on a specific blockchain state.
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
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[FAIL] package_name::call_and_invoke

Failure data:
    original value: [2619239621329578143946475627394146418642347364], converted to a string: [user is not allowed]

Tests: 0 passed, 1 failed, 0 skipped

Failures:
    package_name::call_and_invoke
```

Our user validation is not letting us call the contract, because the default caller address is not `123`.

## Using Cheatcodes in Tests

By using cheatcodes, we can change various properties of transaction info, block info, etc.
For example, we can use the [`start_prank`](../appendix/cheatcodes/start_prank.md) cheatcode to change the caller
address,
so it passes our validation.

### Pranking the Address

```rust
use snforge_std::{ declare, ContractClassTrait, start_prank };

#[test]
fn call_and_invoke() {
    let contract = declare('HelloStarknet');
    let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
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
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] package_name::call_and_invoke
Tests: 1 passed, 0 failed, 0 skipped
```

### Canceling the Prank

Most cheatcodes come with corresponding `start_` and `stop_` functions that can be used to start and stop the state
change.
In case of the `start_prank`, we can cancel the address change
using [`stop_prank`](../appendix/cheatcodes/stop_prank.md)

```rust
use snforge_std::stop_prank;

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
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[FAIL] package_name::call_and_invoke

Failure data:
    original value: [2619239621329578143946475627394146418642347364], converted to a string: [user is not allowed]

Tests: 0 passed, 1 failed, 0 skipped

Failures:
    package_name::call_and_invoke
```

### Pranking the constructor

Most of the cheatcodes like `prank`, `mock_call`, `warp`, `roll` do work in the constructor of the contracts.

Let's say, that you have a contract that saves the caller address (deployer) in the constructor, and you want it to be pre-set to a certain value.

To `prank` the constructor, you need to `start_prank` before it is invoked, with the right address. 
To achieve this, you need to precalculate address of the contract using `precalculate_address` of `ContractClassTrait` on declared contract,
and then use it in `start_prank` as an argument:


```rust
use snforge_std::{ declare, ContractClassTrait, start_prank };

#[test]
fn prank_the_constructor() {
    let contract = declare('HelloStarknet');
    let constructor_arguments = @ArrayTrait::new();
    
    // Precalculate the address to obtain the contract address before the constructor call (deploy) itself
    let contract_address = contract.precalculate_address(constructor_arguments); 
    
    // Change the caller address to 123 before the call to contract.deploy
    start_prank(contract_address, 123.try_into().unwrap());
    
    // The constructor will have 123 set as the caller address 
    contract.deploy(constructor_arguments).unwrap();
}
```

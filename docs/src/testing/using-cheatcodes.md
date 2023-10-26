# Using Cheatcodes

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency)
> using appropriate release tag.
>
> ```toml
> [dependencies]
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
    fn get_block_number(self: @TContractState) -> u64;
    fn get_block_timestamp(self: @TContractState) -> u64;
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

    #[external(v0)]
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
        fn get_block_number(self: @ContractState) -> u64 {
            self.blk_nb.read()
        }
        // Gets the block timestamp
        fn get_block_timestamp(self: @ContractState) -> u64 {
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
    original value: [2619239621329578143946475627394146418642347364], converted to a string: [user is not allowed]

Tests: 0 passed, 1 failed, 0 skipped

Failures:
    tests::call_and_invoke
```

Our user validation is not letting us call the contract, because the default caller address is not `123`.

## Using Cheatcodes in Tests

By using cheatcodes, we can change various properties of transaction info, block info, etc.
For example, we can use the [`start_prank`](../appendix/cheatcodes/start_prank.md) cheatcode to change the caller
address, so it passes our validation.

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
$ snforge test
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] tests::call_and_invoke
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
$ snforge test
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[FAIL] tests::call_and_invoke

Failure data:
    original value: [2619239621329578143946475627394146418642347364], converted to a string: [user is not allowed]

Tests: 0 passed, 1 failed, 0 skipped

Failures:
    tests::call_and_invoke
```

### Mocking the constructor with `prank`

Most of the cheatcodes like `prank`, `mock_call`, `warp`, `roll` do work in the constructor of the contracts.

Let's say, that you have a contract that saves the caller address (deployer) in the constructor, and you want it to be pre-set to a certain value.

To `prank` the constructor, you need to `start_prank` before it is invoked, with the right address. To achieve this, you need to precalculate the address of the contract by using the `precalculate_address` function of `ContractClassTrait` on the declared contract, and then use it in `start_prank` as an argument:

```rust
use snforge_std::{ declare, ContractClassTrait, start_prank };

#[test]
fn mock_constructor_with_prank() {
    // declaring contract
    let contract = declare('HelloStarknet');
    // Precalculate the address to obtain the contract address before the constructor call (deploy) itself
    let contract_address = contract.precalculate_address(@ArrayTrait::new());
    // Change the caller address to 123 before the call to contract.deploy
    start_prank(contract_address, 123.try_into().unwrap());

    // The constructor will have 123 set as the caller address
    contract.deploy(constructor_arguments).unwrap();
}
```

### Mocking the constructor with `roll`

Similarly, as we've seen with the `prank` cheatcode, you can also use the `roll` cheatcode to mock the constructor and pre-set the block number to a specific value, which will be stored in the `blk_number` variable. To do this, you'll need to use the `start_roll` cheatcode, which requires two input parameters: `contract_address` to specify the target contract and `block_number` to set the desired value.

The `contract_address` can be derived by precalculating the address with `precalculate_address` function of theof `ContractClassTrait` on declared contract, as demonstrated in the `prank` example.

It's important to note that the `start_roll` cheatcode needs to be used before deploying the contract to successfully mock the constructor.

```rust
#[test]
fn mock_constructor_with_roll() {
    // declaring contract
    let contract = declare('HelloStarknet');
    // precalculating the contract address
    let contract_address = contract.precalculate_address(@ArrayTrait::new());
    // set the block number to 234 for the precalculated address
    start_roll(contract_address, 234);
    // deploying contract
    let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
    // set dispatcher
    let dispatcher = IHelloStarknetDispatcher { contract_address };
    // retrieving the block number
    let block_number = dispatcher.get_block_number();
    // asserting if block number is 234 set by start_roll()
    assert(block_number == 234, 'Wrong block number');
}
```

### Mocking the constructor with `warp`

As we have seen previously, to mock the constructor with the `warp` cheatcode, you need to use the `start_warp` cheatcode, which requires two input parameters: `contract_address` for specifying the target contract and `block_timestamp` for setting the desired value.

It's important to note that the `start_warp` cheatcode needs to be used before deploying the contract to successfully mock the constructor.

```rust
#[test]
fn mock_constructor_with_warp() {
    // declaring contract
    let contract = declare('HelloStarknet');
    // precalculating the contract address
    let contract_address = contract.precalculate_address(@ArrayTrait::new());
    // set the block timestamp to 1000 for the precalculated address
    start_warp(contract_address, 1000);
    // deploying contract
    let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
    // set dispatcher
    let dispatcher = IHelloStarknetDispatcher { contract_address };
    // retrieving the block timestamp
    let block_timestamp = dispatcher.get_block_timestamp();
    // asserting if block timestamp is 1000 set by start_warp
    assert(block_timestamp == 1000, 'Wrong block timestamp');
}
```

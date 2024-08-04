# Testing Smart Contracts

> ℹ️ **Info**
> 
> To use the library functions designed for testing smart contracts,
> you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using appropriate release tag.
>```toml
> [dev-dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
> ```

Using unit testing as much as possible is a good practice, as it makes your test suites run faster. However, when
writing smart contracts, you often want to test their interactions with the blockchain state and with other contracts.

## The Test Contract

In this tutorial we will be using this Starknet contract, in a new `using_dispatchers` package.

```rust
#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[abi(embed_v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // Increases the balance by the given amount.
        fn increase_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(self.balance.read() + amount);
        }

        // Gets the balance.
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
    }
}
```

Note that the name after `mod` will be used as the contract name for testing purposes.

## Writing Tests

Let's write a test that will deploy the `HelloStarknet` contract and call some functions.

```rust
use snforge_std::{ declare, ContractClassTrait };
use using_dispatchers::{ IHelloStarknetDispatcher, IHelloStarknetDispatcherTrait };

#[test]
fn call_and_invoke() {
    // First declare and deploy a contract
    let contract = declare("HelloStarknet").unwrap();
    // Alternatively we could use `deploy_syscall` here
    let (contract_address, _) = contract.deploy(@array![]).unwrap();

    // Create a Dispatcher object that will allow interacting with the deployed contract
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    // Call a view function of the contract
    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    // Call a function of the contract
    // Here we mutate the state of the storage
    dispatcher.increase_balance(100);

    // Check that transaction took effect
    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
```

> 📝 **Note**
> 
> Notice that the arguments to the contract's constructor (the `deploy`'s `calldata` argument) need to be serialized with `Serde`.
> 
> `HelloStarknet` contract has no constructor, so the calldata remains empty in the example above.

```shell
$ snforge test
Collected 1 test(s) from using_dispatchers package
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
#[starknet::interface]
trait IHelloStarknet<TContractState> {
    // ...
    fn do_a_panic(self: @TContractState);
    fn do_a_string_panic(self: @TContractState);
}

#[starknet::contract]
mod HelloStarknet {
    use array::ArrayTrait;

    // ...

    #[abi(embed_v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // ...

        // Panics
        fn do_a_panic(self: @ContractState) {
            panic(array!['PANIC', 'DAYTAH']);
        }

        fn do_a_string_panic(self: @ContractState) {
            // A macro which allows panicking with a ByteArray (string) instance
            panic!("This is panicking with a string, which can be longer than 31 characters");
        }
    }
}
```

If we called this function in a test, it would result in a failure.

```rust
#[test]
fn failing() {
    let contract = declare("HelloStarknet").unwrap();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    dispatcher.do_a_panic();
}
```

```shell
$ snforge test
Collected 1 test(s) from package_name package
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
// Add those to import safe dispatchers, which are autogenerated, like regular dispatchers
use using_dispatchers::{ IHelloStarknetSafeDispatcher, IHelloStarknetSafeDispatcherTrait };

#[test]
#[feature("safe_dispatcher")]
fn handling_errors() {
    // ...
    let contract = declare("HelloStarknet").unwrap();
    let (contract_address, _) = contract.deploy(@calldata).unwrap();
    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    match safe_dispatcher.do_a_panic() {
        Result::Ok(_) => panic!("Entrypoint did not panic"),
        Result::Err(panic_data) => {
            assert(*panic_data.at(0) == 'PANIC', *panic_data.at(0));
            assert(*panic_data.at(1) == 'DAYTAH', *panic_data.at(1));
        }
    };
}
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
// Necessary utility function import
use snforge_std::byte_array::try_deserialize_bytearray_error;

#[test]
#[feature("safe_dispatcher")]
fn handling_string_errors() {
    // ...
    let contract = declare("HelloStarknet").unwrap();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };
    
    match safe_dispatcher.do_a_string_panic() {
        Result::Ok(_) => panic!("Entrypoint did not panic"),
        Result::Err(panic_data) => {
            let str_err = try_deserialize_bytearray_error(panic_data.span()).expect('wrong format');
            assert(
                str_err == "This is panicking with a string, which can be longer than 31 characters", 
                'wrong string received'
            );
        }
    };
}
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

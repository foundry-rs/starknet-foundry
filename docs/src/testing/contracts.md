# Testing Smart Contracts

> ℹ️ **Info**
> To use the library functions designed for testing smart contracts,
> you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency)
> using appropriate release tag.
>```toml
> [dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
> ```

Using unit testing as much as possible is a good practice, as it makes your test suites run faster. However, when
writing smart contracts, you often want to test their interactions with the blockchain state and with other contracts.

## The Test Contract

In this tutorial we will be using this Starknet contract

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

#[test]
fn call_and_invoke() {
    // First declare and deploy a contract
    let contract = declare('HelloStarknet');
    // Alternatively we could use `deploy_syscall` here
    let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();

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

> ⚠️ **Warning**
>
> As of Cairo 2.5.0, the `SafeDispatcher` need special marking with `#[feature("safe_dispatcher")]` before **each** call made with it.
> For the detailed explanation of this behavior, refer to [the shamans post](https://community.starknet.io/t/cairo-v2-5-0-is-out/112807#safe-dispatchers-15).
> For the implementation, check the example below.


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
            let mut arr = ArrayTrait::new();
            arr.append('PANIC');
            arr.append('DAYTAH');
            panic(arr);
        }

        fn do_a_string_panic(self: @ContractState) {
            assert!(false, "This a panicking with a string, which can be longer");
        }
    }
}
```

If we called this function in a test, it would result in a failure.

```rust
#[test]
fn failing() {
    // ...

    let contract_address = contract.deploy(@calldata).unwrap();
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
    original value: [344693033283], converted to a string: [PANIC]
    original value: [75047462256968], converted to a string: [DAYTAH]

Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

Failures:
    tests::failing
```

### `SafeDispatcher`

Using `SafeDispatcher` we can test that the function in fact panics with an expected message.

```rust
#[test]
fn handling_errors() {
    // ...

    let contract_address = contract.deploy(@calldata).unwrap();
    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };
    
    #[feature("safe_dispatcher")] // Mandatory tag since cairo 2.5.0
    match safe_dispatcher.do_a_panic() {
        Result::Ok(_) => panic_with_felt252('shouldve panicked'),
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

Similarly, you can handle the panics which use string as an argument (like an `assert!` macro)

```rust
// Necessary struct and trait imports for string errors mapping
use snforge_std::errors::{ SyscallResultStringErrorTrait, PanicDataOrString };
// ...
#[test]
fn handling_string_errors() {
    // ...
    let contract_address = contract.deploy(@calldata).unwrap();
    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };
    
    // Notice the `map_string_error` helper usage here, and different error type
    #[feature("safe_dispatcher")]
    match safe_dispatcher.do_a_string_panic().map_string_error() { 
        Result::Ok(_) => panic_with_felt252('shouldve panicked'),
        Result::Err(panic_data) => {
            match x { 
                PanicDataOrString::PanicData(_) => panic_with_felt252('wrong format'),
                PanicDataOrString::String(str) => {
                    assert(
                        str == "This a panicking with a string, which can be longer", 
                        'wrong string received'
                    );
                }
            }
        }
    };
}
```
You also could skip the de-serialization of the `panic_data`, and not use `map_string_error`, but this way you can actually use assertions on the `ByteArray` that was used to panic. 

### Expecting Test Failure

Sometimes the test code failing can be a desired behavior.
Instead of manually handling it, you can simply mark your test as `#[should_panic(...)]`.
[See here](./testing.md#expected-failures) for more details.

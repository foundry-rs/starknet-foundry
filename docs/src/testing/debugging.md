# Debugging

> ℹ️ **Info**
> To use `PrintTrait` you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency) 
> using appropriate release tag.
>```toml
> [dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.5.0" }
> ```

Starknet Foundry standard library adds a utility method for printing inside tests and contracts to facilitate simple debugging.

## In tests

Here's a test with example use of print method:

```rust
// Make sure to import Starknet Foundry PrintTrait
use array::ArrayTrait;
use snforge_std::io::PrintTrait;

#[test]
fn test_print() {
    'Print short string:'.print();
    'my string'.print();

    'Print number:'.print();
    123.print();

    'Print array:'.print();
    let mut arr = ArrayTrait::new();
    arr.append(1);
    arr.append(2);
    arr.append(3);
    arr.print();

    'Print bool:'.print();
    (1 == 5).print();
    assert(1 == 1, 'simple check');
}
```

Running tests will include prints in the output:

```shell
$ snforge
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
original value: [1794026292945241370577200538206453096157964090], converted to a string: [Print short string:]
original value: [2019423207056158060135], converted to a string: [my string]
original value: [6373661751074312243962081276474], converted to a string: [Print number:]
original value: [123], converted to a string: [{]
original value: [97254360215367257408299385], converted to a string: [Print array:]
original value: [1], converted to a string: []
original value: [2], converted to a string: []
original value: [3], converted to a string: []
original value: [379899844591278365831020], converted to a string: [Print bool:]
original value: [439721161573], converted to a string: [false]
[PASS] tests::test_print::test_print
Tests: 1 passed, 0 failed, 0 skipped
```

Forge tries to convert values to strings when possible. In case conversion is not possible,
just `original value` is printed.

## In contracts
> ⚠️ **Warning**
> 
> - Make sure to remove all of the prints before compiling the final version of your contract. 
> - This feature is highly experimental and breaking changes are to be expected.

Here is an example contract:

```rust
#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod HelloStarknet {
    // Note: PrintTrait has to be imported
    use snforge_std::io::PrintTrait;

    #[storage]
    struct Storage {
        balance: felt252, 
    }

    #[external(v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        fn increase_balance(ref self: ContractState, amount: felt252) {
            assert(amount != 0, 'Amount cannot be 0');
            self.balance.write(self.balance.read() + amount);
            'The new balance is:'.print();
            self.balance.read().print();
        }

        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
    }
}
```
With a test:
```rust
#[test]
fn test_increase_balance() {
    let contract_address = deploy_contract('HelloStarknet');

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}
```
We get the following output:
```
$ snforge                                                                                              
    Updating [...]
   Compiling [...]
warn: libfunc `cheatcode` is not allowed in the libfuncs list `Default libfunc list`
 --> contract: HelloStarknet
help: try compiling with the `experimental` list
 --> Scarb.toml
    [[target.starknet-contract]]
    allowed-libfuncs-list.name = "experimental"

    Finished release target(s) in 2 seconds


Collected 2 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
original value: [1882356686041040905424961122938381530884043578], converted to a string: [The new balance is:]
original value: [42], converted to a string: [*]
[PASS] tests::test_contract::test_increase_balance
Tests: 1 passed, 0 failed, 0 skipped
```

**Note**: 
- the print output shows **above** the test, this may change in the future.
- the warning is to be expected as the prints should be removed after debugging.

> ℹ️ **Info**
> More debugging features will be added to Starknet Foundry once Starknet compiler implements necessary support.

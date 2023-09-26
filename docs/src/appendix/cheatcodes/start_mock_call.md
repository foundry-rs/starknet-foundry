# `start_mock_call`

> `fn start_mock_call<T, impl TSerde: serde::Serde<T>, impl TDestruct: Destruct<T>>(contract_address: ContractAddress, function_name: felt252, ret_data: T)`

Mocks contract call to a `function_name` of a contract at the given address. A call to function `function_name` will
return data provided in `ret_data` argument.

An address with no contract can be mocked as well. Mock can be canceled with [`stop_mock_call`](./stop_mock_call.md).

- `contract_address` - target contract address
- `function_name` - name of the function in a contract at the `contract_address` that will be mocked
- `ret_data` - data to return by the function `function_name`

> 📝 **Note**
> Mocks do not have any effect on function calls withing the contract itself.
> E.g. for a function within a contract defined like this:
>
> ```rust
> fn function_a(self: @ContractState) {
>   function_b()
> } 
> ```
>
> Mocking `function_b` would have no effect.

For contract implementation:

```rust
// ...
#[external(v0)]
impl IContractImpl of IContract<ContractState> {
    #[storage]
    struct Storage {
        // ...
        
        balance: felt252
    }

    fn set_balance(ref self: ContractState, new_balance: felt252) {
        self.balance.write(new_balance);
    }

    fn get_balance(self: @ContractState) -> felt252 {
        self.balance.read()
    }
}
// ...
```

We can use `start_mock_call` in a test to change the data returned by `get_balance` for a given contract:

```rust
use snforge_std::start_mock_call;

#[test]
fn test_mock_call() {
    // ...

    let mock_ret_data = 421;
    start_mock_call(contract_address, 'get_balance', mock_ret_data);

    dispatcher.set_balance(13);
    let balance = dispatcher.get_balance();
    assert(balance == 421, 'Wrong balance'); // this assert passes
}
```

## Mocking non-existent functions

It is also possible to simulate contract having a specific method by mocking a non-existent selector.

Let's assume we have defined an interface to use a dispatcher, but this interface contains a function that is not
actually defined in actually deployed contract:
actually defined in a deployed contract:

```rust
// ...
// Assume we define an interface like that. 
// However, it contains a function that does not actually exist on the implementing contract.
// That is IOtherContract used a different interface that did not define
// `function_not_actually_implemented`.
// 
// Normally calling `function_not_actually_implemented` would fail.
#[starknet::interface]
impl IOtherContract<TContractState> {
    fn function_implemented(self: @TContractState) -> felt252;
    fn function_not_actually_implemented(self: @TContractState) -> felt252;
}

#[external(v0)]
impl IContractImpl of IContract<ContractState> {
    fn call_not_actually_implemented(self: @ContractState) -> felt252 {
        // ...
        let other_contract_dispatcher = IOtherContractDispatcher { contract_address };
        other_contract_dispatcher.function_not_actually_implemented()
    }
}
```

This test would then fail, because `function_not_actually_implemented` does not exist.

```rust
#[test]
fn test_mock_not_implemented() {
    // ...

    let result = dispatcher.call_not_actually_implemented();
    // ...
}
```

We can, however, mock this function, even though it is not implemented. This test will pass:

```rust
#[test]
fn test_mock_not_implemented() {
    // ...

    let mock_ret_data = 42;
    start_mock_call(contract_address, 'function_not_actually_implemented', mock_ret_data);

    let result = dispatcher.call_not_actually_implemented();
    assert(result == 42, 'result != 42');
    // ...
}
```


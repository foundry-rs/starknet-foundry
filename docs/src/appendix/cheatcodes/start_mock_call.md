# `start_mock_call`

> `fn start_mock_call<T, impl TSerde: serde::Serde<T>, impl TDestruct: Destruct<T>>(contract_address: ContractAddress, function_name: felt252, ret_data: T)`

Mocks contract call to a `function_name` of a contract at the given address. A call to function `function_name` will return data provided in `ret_data` argument. 

If there is a contract deployed at the given address, mocked function won't be executed. Address with no contract can be mocked as well.
Mock can be canceled with [`stop_mock_call`](./stop_mock_call.md).

- `contract_address` - target contract address
- `function_name` - name of the function in a contract at the `contract_address` that will be mocked
- `ret_data` - data to return by the function `function_name`

> 📝 **Note**
> The inner call (i.e. when a contract calls a function from within itself) cannot be mocked.

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
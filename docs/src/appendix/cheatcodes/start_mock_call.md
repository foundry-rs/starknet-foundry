# `start_mock_call`

> `fn start_mock_call(contract_address: ContractAddress, function_name: felt252, ret_data: Array::<felt252>)`

Mocks external function `function_name` for a contract at the given address. The function `function_name` will return data provided in `ret_data` argument.
Mock can be canceled with [`stop_mock_call`](./stop_mock_call.md).

- `contract_address` - target contract address
- `function_name` - name of the function in a contract at the `contract_address` that will be mocked
- `ret_data` - data to return by the function `function_name`

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
fn test_mock() {
    // ...
    
    let mut mock_ret_data = ArrayTrait::new();
    mock_ret_data.append(421);
    start_mock_call(contract_address, 'get_balance', mock_ret_data);

    dispatcher.set_balance(13);
    let balance = dispatcher.get_balance();
    assert(balance == 421, 'Wrong balance'); // this assert passes
}
```
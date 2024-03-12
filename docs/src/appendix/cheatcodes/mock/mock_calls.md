# `mock_calls`

> `fn mock_calls<T, impl TSerde: serde::Serde<T>, impl TDestruct: Destruct<T>>(contract_address: ContractAddress, function_selector: felt252, ret_data: T, amount: u32)`

Mocks `amount` contract calls to a `function_selector` of a contract for the given address.
A call to function `function_selector` will return data provided in `ret_data` argument.

An address with no contract can be mocked as well.

Mock can be canceled with [`stop_mock_call`](./stop_mock_call.md).

- `contract_address` - target contract address
- `function_selector` - selector of the function in a contract at the `contract_address` that will be mocked
- `ret_data` - data to return by the function `function_selector`
- `amount` - amount of calls to mock the function for

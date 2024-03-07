# `mock_call`

> `fn mock_call<T, impl TSerde: serde::Serde<T>, impl TDestruct: Destruct<T>>(contract_address: ContractAddress, function_selector: felt252, ret_data: T, span: CheatSpan)`

Mocks contract call to a `function_selector` of a contract at the given address, for a given duration. A call to function `function_selector` will
return data provided in `ret_data` argument.

An address with no contract can be mocked as well. Mock can be canceled with [`stop_mock_call`](./stop_mock_call.md).

- `contract_address` - target contract address
- `function_selector` - selector of the function in a contract at the `contract_address` that will be mocked
- `ret_data` - data to return by the function `function_selector`
- `span` - instance of [`CheatSpan`](../cheat_span.md) specifying the duration of mock

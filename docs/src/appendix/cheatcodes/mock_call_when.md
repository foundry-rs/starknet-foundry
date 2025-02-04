# `mock_call_when`

Cheatcodes mocking contract entry point calls:

## `MockCallData`

```rust
pub enum MockCallData {
    Any,
    Values: Span<felt252>,
}
```

`MockCallData` is an enum used to specify for which call data the contract entry point will be mocked.
- `Any` mock the contract entry point for any call data.
- `Values` mock the contract entry point only for this call data.

## `mock_call_when`
> `fn mock_call_when<T, impl TSerde: serde::Serde<T>, impl TDestruct: Destruct<T>>(
>   contract_address: ContractAddress, function_selector: felt252, call_data: MockCallData, ret_data: T, n_times: u32
> )`

Mocks contract call to a `function_selector` of a contract at the given address, with the given call data, for `n_times` first calls that are made 
to the contract. 
A call to function `function_selector` will return data provided in `ret_data` argument. 
An address with no contract can be mocked as well. 
An entrypoint that is not present on the deployed contract is also possible to mock.
Note that the function is not meant for mocking internal calls - it works only for contract entry points.

## `start_mock_call_when`
> `fn start_mock_call<T, impl TSerde: serde::Serde<T>, impl TDestruct: Destruct<T>>(
>   contract_address: ContractAddress, function_selector: felt252, call_data: MockCallData, ret_data: T
> )`

Mocks contract call to a `function_selector` of a contract at the given address, with the given call data, indefinitely.
See `mock_call_when` for comprehensive definition of how it can be used.


### `stop_mock_call_when`

> `fn stop_mock_call_when(contract_address: ContractAddress, function_selector: felt252, call_data: MockCallData)`

Cancels the `mock_call_when` / `start_mock_call_when` for the function `function_selector` of a contract at the given addressn with the given call data

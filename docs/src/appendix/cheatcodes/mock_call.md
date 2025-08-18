# `mock_call`

Cheatcodes mocking contract entry point calls:

## `mock_call`
> `fn mock_call<T, impl TSerde: serde::Serde<T>, impl TDestruct: Destruct<T>>(
>   contract_address: ContractAddress, function_selector: felt252, ret_data: T, n_times: u32
> )`

Mocks contract call to a `function_selector` of a contract at the given address, for `n_times` first calls that are made to the contract. 
A call to function `function_selector` will return data provided in `ret_data` argument. 
An address with no contract can be mocked as well. 
An entrypoint that is not present on the deployed contract is also possible to mock.
Note that the function is not meant for mocking internal calls - it works only for contract entry points.

## `start_mock_call`
> `fn start_mock_call<T, impl TSerde: serde::Serde<T>, impl TDestruct: Destruct<T>>(
>   contract_address: ContractAddress, function_selector: felt252, ret_data: T
> )`

Mocks contract call to a `function_selector` of a contract at the given address, indefinitely.
See `mock_call` for comprehensive definition of how it can be used.


## `stop_mock_call`

> `fn stop_mock_call(contract_address: ContractAddress, function_selector: felt252)`

Cancels the `mock_call` / `start_mock_call` for the function `function_selector` of a contract at the given address.

## Example

Let's consider a contract which simulates a shopping cart. It has a function `get_products` that returns a list of products in the cart.

```rust
{{#include ../../../listings/cheatcodes_reference/src/mock_call_example.cairo}}
```

Test code:

```rust
{{#include ../../../listings/cheatcodes_reference/tests/test_mock_call.cairo}}
```

Let's run the test:

<!-- { "package_name": "cheatcodes_reference", "scarb": ">=2.11.4" } -->
```shell
$ snforge test test_mock_call
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from cheatcodes_reference package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] cheatcodes_reference_integrationtest::test_mock_call::test_mock_call ([..])
Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>
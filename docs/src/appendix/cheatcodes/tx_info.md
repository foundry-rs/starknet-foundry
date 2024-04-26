# `tx_info`

Cheatcodes modifying `tx_info`:

## `spoof`

> `fn spoof(target: CheatTarget, tx_info_mock: TxInfoMock, span: CheatSpan)`

Changes `TxInfo` returned by `get_tx_info()` for the targeted contract and span.

## `start_spoof`

> `fn start_spoof(target: CheatTarget, tx_info_mock: TxInfoMock)`

Changes `TxInfo` returned by `get_tx_info()` for the targeted contract until the spoof is canceled with `stop_spoof`.

## `stop_spoof`

> `fn stop_spoof(target: CheatTarget)`

Cancels the `spoof` / `start_spoof` for the given target.


## `TxInfoMock` 

A structure used for setting individual fields in `TxInfo`
All fields are optional, with optional value meaning as defined:
- `None` means that the field is going to be reset to the initial value
- `Some(n)` means that the value will be set to the `n` value
```
struct TxInfoMock {
    version: Option<felt252>,
    account_contract_address: Option<ContractAddress>,
    max_fee: Option<u128>,
    signature: Option<Span<felt252>>,
    transaction_hash: Option<felt252>,
    chain_id: Option<felt252>,
    nonce: Option<felt252>,
    // starknet::info::v2::TxInfo fields
    resource_bounds: Option<Span<starknet::info::v2::ResourceBounds>>,
    tip: Option<u128>,
    paymaster_data: Option<Span<felt252>>,
    nonce_data_availability_mode: Option<u32>,
    fee_data_availability_mode: Option<u32>,
    account_deployment_data: Option<Span<felt252>>,
}
```

### `starknet::info::v2::ResourceBounds`
```
pub struct ResourceBounds {
    resource: felt252,
    max_amount: u64,
    max_price_per_unit: u128,
}
```
A struct responsible for setting the resource bounds, used in `TxInfoMock`.

## `TxInfoMockTrait`
```
trait TxInfoMockTrait {
    fn default() -> TxInfoMock;
}
```

Returns a default object initialized with `Option::None` for each field.
Useful for setting only a few of the fields instead of all of them.
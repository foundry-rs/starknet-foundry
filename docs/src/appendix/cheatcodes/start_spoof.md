# `start_spoof`

> `fn start_spoof(target: CheatTarget, tx_info_mock: TxInfoMock)`

Changes `TxInfo` returned by `get_tx_info()` for the targeted contract until the spoof is stopped
with [stop_spoof](./stop_spoof.md).

- `target` - instance of [`CheatTarget`](./cheat_target.md) specifying which contracts to spoof
- `TxInfoMock` - a struct with same structure as `TxInfo` (returned by `get_tx_info()`), 

To mock the field of `TxInfo`, set the corresponding field of `TxInfoMock` to `Some(mocked_value)`. Setting the field to `None` will use a default value - the field will not be mocked. Using `None` will also cancel current mock for that field. See below for practical example.

> 📝 **Note**
>
> To get access to fields from `starknet::info::v2::TxInfo`, you can use
> `get_execution_info_v2_syscall().unwrap_syscall().unbox().tx_info.unbox()`

```rust
struct TxInfoMock {
    version: Option<felt252>,
    account_contract_address: Option<ContractAddress>,
    max_fee: Option<u128>,
    signature: Option<Span<felt252>>,
    transaction_hash: Option<felt252>,
    chain_id: Option<felt252>,
    nonce: Option<felt252>,
    // starknet::info::v2::TxInfo fields
    resource_bounds: Option<Span<ResourceBounds>>,
    tip: Option<u128>,
    paymaster_data: Option<Span<felt252>>,
    nonce_data_availability_mode: Option<u32>,
    fee_data_availability_mode: Option<u32>,
    account_deployment_data: Option<Span<felt252>>,
}

trait TxInfoMockTrait {
    // Returns a default object initialized with Option::None for each field  
    fn default() -> TxInfoMock;
}
```

For contract implementation:

```rust
// ...
#[storage]
struct Storage {
    stored_hash: felt252
}

#[abi(embed_v0)]
impl IContractImpl of IContract<ContractState> {
    fn store_tx_hash(ref self: ContractState) {
        let tx_info = get_tx_info().unbox();
        self.stored_hash.write(tx_info.transaction_hash);
    }

    fn get_stored_tx_hash(self: @ContractState) -> felt252 {
        self.stored_hash.read()
    }
}
// ...
```

```rust
use snforge_std::{ start_spoof, CheatTarget };

#[test]
fn test_spoof() {
    // ...
    let tx_hash_before_mock = dispatcher.get_stored_tx_hash();
    
    // Change transaction_hash to 1234
    // All other fields of `TxInfo` remain unchanged
    let mut tx_info = TxInfoMockTrait::default();
    tx_info.transaction_hash = Option::Some(1234);
    
    start_spoof(CheatTarget::One(contract_address), tx_info);
    
    dispatcher.store_tx_hash();
    
    let tx_hash = dispatcher.get_stored_tx_hash();
    assert(tx_hash == 1234, 'tx_hash should be mocked');
    
    // Cancel tx_info.transaction_hash mocking by setting the field to `None`
    // All other fields of `TxInfo` remain unchanged
    start_spoof(CheatTarget::One(contract_address), TxInfoMockTrait::default());
    dispatcher.store_tx_hash();
    
    let tx_hash = dispatcher.get_stored_tx_hash();
    assert(tx_hash == tx_hash_before_mock, 'tx_hash was not reverted');
}
```

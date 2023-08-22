# `start_spoof`

> `fn start_spoof(contract_address: ContractAddress, tx_info_mock: TxInfoMock)`
Changes `TxInfo` returned by `get_tx_info()` for the targeted contract until the spoof is stopped
with [stop_spoof](./stop_spoof.md).

- `contract_address` address of the contract for which `get_tx_info()` result will be mocked.
- `TxInfoMock` - a struct with same structure as `TxInfo` (returned by `get_tx_info()`), 

To mock the field of `TxInfo`, set the corresponding field of `TxInfoMock` to `Some(mocked_value)`. Setting the field to `None` will use a default value - the field will not be mocked. Using `None` will also cancel current mock for that field. See below for practical example.

```rust
struct TxInfoMock {
    version: Option<felt252>,
    account_contract_address: Option<felt252>,
    max_fee: Option<u128>,
    signature: Option<Array<felt252>>,
    transaction_hash: Option<felt252>,
    chain_id: Option<felt252>,
    nonce: Option<felt252>,
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

#[external(v0)]
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
use snforge_std::start_spoof;

#[test]
fn test_spoof() {
    // ...
    
    // Change transaction_hash to 1234
    // All other fields of `TxInfo` remain unchanged
    let mut tx_info = TxInfoMockTrait::default();
    tx_info.transaction_hash = Option::Some(1234);
    
    start_spoof(contract_address, tx_info);
    
    dispatcher.store_tx_hash(13);
    
    let tx_hash = dispatcher.get_stored_tx_hash();
    assert(tx_hash == 1234, 'Wrong tx_hash'); // this assert passes
}
```
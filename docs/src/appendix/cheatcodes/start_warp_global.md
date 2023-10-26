# `start_warp_global`

> `fn start_warp_global(block_timestamp: u64)`

Changes the block timestamp for all contracts.
The change can be canceled with [`stop_warp_global`](./stop_warp_global.md).

- `block_timestamp` - block timestamp to be set

**Note:** `start_warp_global` overrides `start_warp`. This means that if `start_warp` has been called
for contract A, and `start_warp_global` has also been called, then calling `get_block_timestamp` in contract A
will return the timestamp set in `start_warp_global`. If `stop_warp_global` then called, then calling `get_block_timestamp`
in contract A will return the timestamp set in `start_warp`. 

For contract implementation:

```rust
// ...
#[storage]
struct Storage {
    // ...
    stored_block_timestamp: u64
}

#[external(v0)]
impl IContractImpl of IContract<ContractState> {
    fn set_block_timestamp(ref self: ContractState) {
        self.stored_block_timestamp.write(starknet::get_block_timestamp());
    }
    
    fn get_block_timestamp(self: @ContractState) -> u64 {
        self.stored_block_timestamp.read()
    }
}
// ...
```

We can use `start_warp` in a test to change the block timestamp for a given contract:

```rust
use snforge_std::start_warp;

#[test]
fn test_warp() {
    // ...

    // `start_warp_global` should override `start_warp`
    start_warp(dispatcher.contract_address, 500);
    start_warp_global(1000);

    dispatcher.set_block_timestamp();
    let new_timestamp = dispatcher.get_block_timestamp();
    assert(new_timestamp == 1000, 'Wrong timestamp');

    stop_warp_global();
    dispatcher.set_block_timestamp();
    let new_timestamp = dispatcher.get_block_timestamp();
    assert(new_timestamp == 500, 'Wrong timestamp');
}
```

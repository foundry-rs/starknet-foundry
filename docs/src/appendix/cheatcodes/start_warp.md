# `start_warp`

> `fn start_warp(contract_address: ContractAddress, block_timestamp: u64)`

Changes the block timestamp for a contract at the given address.
The change can be canceled with [`stop_warp`](./stop_warp.md).

- `contract_address` - target contract address
- `block_timestamp` - block timestamp to be set

For contract implementation:

```rust
// ...
#[external(v0)]
impl IContractImpl of IContract<ContractState> {
    #[storage]
    struct Storage {
        // ...

        stored_block_timestamp: u64
    }
    
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
#[test]
fn test_warp() {
    // ...

    start_warp(contract_address, 1000);

    dispatcher.set_block_timestamp();
    let new_timestamp = dispatcher.get_block_timestamp();
    assert(new_timestamp == 1000, 'Wrong timestamp');
}
```

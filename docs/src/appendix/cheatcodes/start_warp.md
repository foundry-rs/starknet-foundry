# `start_warp`

> `fn start_warp(target: CheatTarget, block_timestamp: u64)`

Changes the block timestamp for the given target.
The change can be canceled with [`stop_warp`](./stop_warp.md).

- `target` - instance of [`CheatTarget`](./cheat_target.md) specifying which contracts to warp
- `block_timestamp` - block timestamp to be set

For contract implementation:

```rust
// ...
#[storage]
struct Storage {
    // ...
    stored_block_timestamp: u64
}

#[abi(embed_v0)]
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

We can use `start_warp` in a test to change the block timestamp for contracts:

```rust
use snforge_std::{start_warp, CheatTarget};

#[test]
fn test_warp() {
    // ...

    start_warp(CheatTarget::One(contract_address), 1000);

    dispatcher.set_block_timestamp();
    let new_timestamp = dispatcher.get_block_timestamp();
    assert(new_timestamp == 1000, 'Wrong timestamp');
}
```

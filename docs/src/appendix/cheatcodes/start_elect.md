# `start_elect`

> `fn start_elect(target: CheatTarget, sequencer_address: ContractAddress)`

Changes the sequencer address for a given target.
The change can be canceled with [`stop_elect`](./stop_elect.md).

- `target` - instance of [`CheatTarget`](./cheat_target.md) specifying which contracts to elect
- `sequencer_address` - sequencer address to be set

For contract implementation:

```rust
// ...
#[storage]
struct Storage {
    stored_sequencer_address: ContractAddress
}

#[abi(embed_v0)]
impl IContractImpl of IContract<ContractState> {
    fn set_sequencer_address(ref self: ContractState) {
        self.stored_sequencer_address.write(starknet::get_block_info().unbox().sequencer_address);
    }
    
    fn get_sequencer_address(self: @ContractState) -> ContractAddress {
        self.stored_sequencer_address.read()
    }
}
```

We can use `start_elect` in a test to change the sequencer address for contracts:

```rust
use snforge_std::{start_elect, CheatTarget};

#[test]
fn test_elect() {
    // ...

    start_elect(CheatTarget::One(contract_address), 234.try_into().unwrap());

    dispatcher.set_sequencer_address();
    let new_sequencer_address = dispatcher.get_sequencer_address();
    assert(new_sequencer_address == 234.try_into().unwrap(), 'Wrong sequencer address');
}
```

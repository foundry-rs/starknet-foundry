# `start_roll`

> `fn start_roll(contract_address: ContractAddress, block_number: u64)`

Changes the block number for a contract at the given address.
The change can be canceled with [`stop_roll`](./stop_roll.md).

- `contract_address` - target contract address
- `block_number` - block number to be set

For contract implementation:

```rust
// ...
#[storage]
struct Storage {
    // ...
    stored_block_number: u64
}

#[external(v0)]
impl IContractImpl of IContract<ContractState> {
    fn set_block_number(ref self: ContractState) {
        self.stored_block_number.write(starknet::get_block_info().unbox().block_number);
    }
    
    fn get_block_number(self: @ContractState) -> u64 {
        self.stored_block_number.read()
    }
}
// ...
```

We can use `start_roll` in a test to change the block number for a given contract:

```rust
use snforge_std::start_roll;

#[test]
fn test_roll() {
    // ...

    start_roll(contract_address, 234);

    dispatcher.set_block_number();
    let new_block_number = dispatcher.get_block_number();
    assert(new_block_number == 234, 'Wrong block number');
}
```

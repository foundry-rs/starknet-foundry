# `stop_roll`

> `fn stop(contract_address: ContractAddress)`

Cancels block number mock for the contract at the given address.

- `contract_address` - target contract address


For a contract implementation
```rust
// ...
#[external(v0)]
impl IRollChecker of super::IRollChecker<ContractState> {
    fn get_block_number(ref self: ContractState) -> u64 {
        starknet::get_block_info().unbox().block_number
    }
}
// ...
```

We can use roll in a test to change 
```rust
use result::ResultTrait;

#[test]
fn test_roll() {
    // ...
    let original_block_number = dispatcher.get_block_number().unwrap();
    start_roll(contract_address, 234);

    let new_block_number = dispatcher.get_block_number().unwrap();
    assert(new_block_number == 234, 'Wrong block number');

    stop_roll(contract_address);

    // Block number is not mocked anymore
    let current_block_number = dispatcher.get_block_number().unwrap();
    assert(current_block_number == original_block_number, 'Wrong block number');
    // ...
}
```

# `start_roll`

> `fn start_roll(contract_address: ContractAddress, block_number: u64)`

Mocks block number for a contract at the given address. It persists until [`stop_roll`](./stop_roll.md) is called.

- `contract_address` - target contract address
- `block_number` - mocked value of block number


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
use array::ArrayTrait;
use option::OptionTrait;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;
use cheatcodes::PreparedContract;
use forge_print::PrintTrait;

#[test]
fn test_roll() {
    // ...
    start_roll(contract_address, 234);

    let new_block_number = dispatcher.get_block_number();
    assert(new_block_number == 234, 'Wrong block number');

    stop_roll(contract_address);

    // ...
}
```

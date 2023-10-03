# `start_warp`

> `fn start_warp(contract_address: ContractAddress, block_timestamp: u64)`

Changes the block timestamp for a contract at the given address.
The change can be canceled with [`stop_warp`](./stop_warp.md).

- `contract_address` - target contract address
- `block_timestamp` - block timestamp to be set

For contract implementation:

```rust
#[starknet::interface]
trait IConstructorWarpChecker<TContractState> {
    fn get_stored_block_timestamp(ref self: TContractState) -> u64;
}

#[starknet::contract]
mod ConstructorWarpChecker {
    use box::BoxTrait;
    #[storage]
    struct Storage {
        blk_timestamp: u64,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let blk_timestamp = starknet::get_block_info().unbox().block_timestamp;
        self.blk_timestamp.write(blk_timestamp);
    }

    #[external(v0)]
    impl IConstructorWarpChecker of super::IConstructorWarpChecker<ContractState> {
        fn get_stored_block_timestamp(ref self: ContractState) -> u64 {
            self.blk_timestamp.read()
        }
    }
}
```

We can use `start_warp` in a test to change the block timestamp for a given contract. In the example below, we utilize `start_warp` to mock the constructor by setting the block timestamp to `1000`. This is achieved by precalculating the contract's address in the `deploy_contract` function and setting the block timestamp for the precomputed contract.

```rust
use array::ArrayTrait;
use starknet::ContractAddress;

use snforge_std::{declare, ContractClassTrait};
use snforge_std::start_warp;

fn deploy_contract(name: felt252) -> ContractAddress {
    // declaring contract
    let contract = declare(name);
    // precalculating the contract address
    let contract_address = contract.precalculate_address(@ArrayTrait::new());
    // setting the block timestamp with the precalculated address
    start_warp(contract_address, 1000);
    // deploying contract
    contract.deploy(@ArrayTrait::new()).unwrap()
}

#[test]
fn test_warp() {
    // deploy contract
    let contract_address = deploy_contract('ConstructorWarpChecker');
    // set dispatcher
    let dispatcher = IConstructorWarpCheckerDispatcher { contract_address };
    // retrieving the block timestamp
    let block_timestamp = dispatcher.get_stored_block_timestamp();
    // asserting if block timestamp is 1000 set by start_warp
    assert(block_timestamp == 1000, 'Wrong block timestamp');
}
```

# `start_roll`

> `fn start_roll(contract_address: ContractAddress, block_number: u64)`

Changes the block number for a contract at the given address.
The change can be canceled with [`stop_roll`](./stop_roll.md).

- `contract_address` - target contract address
- `block_number` - block number to be set

For contract implementation:

```rust
#[starknet::interface]
trait IConstructorRollChecker<TContractState> {
    fn get_stored_block_number(ref self: TContractState) -> u64;
}

#[starknet::contract]
mod ConstructorRollChecker {
    use box::BoxTrait;
    #[storage]
    struct Storage {
        blk_nb: u64,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let block_numb = starknet::get_block_info().unbox().block_number;
        self.blk_nb.write(block_numb);
    }

    #[external(v0)]
    impl IConstructorRollChecker of super::IConstructorRollChecker<ContractState> {
        fn get_stored_block_number(ref self: ContractState) -> u64 {
            self.blk_nb.read()
        }
    }
}
```

We can use `start_roll` in a test to modify the block number for a given contract. In the example below, we utilize `start_roll` to mock the constructor by setting the block number to `234`. This is achieved by precalculating the contract's address in the `deploy_contract` function and setting the block number for the precomputed contract.

```rust
use array::ArrayTrait;
use starknet::ContractAddress;

use snforge_std::{declare, ContractClassTrait};
use snforge_std::start_roll;

fn deploy_contract(name: felt252) -> ContractAddress {
    // declaring contract
    let contract = declare(name);
    // precalculate the contract address
    let contract_address = contract.precalculate_address(@ArrayTrait::new());
    // setting the block number with the precalculated address
    start_roll(contract_address, 234);
    // deploying contract
    contract.deploy(@ArrayTrait::new()).unwrap()
}

#[test]
fn test_roll() {
    // deploy contract
    let contract_address = deploy_contract('ConstructorRollChecker');
    // set dispatcher
    let dispatcher = IConstructorRollCheckerDispatcher { contract_address };
    // retrieving the block number
    let block_number = dispatcher.get_stored_block_number();
    // asserting if block number is correctly mocked by start_roll
    assert(block_number == 234, 'Wrong block number');
}
```

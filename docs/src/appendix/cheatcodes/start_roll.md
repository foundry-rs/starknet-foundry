# `start_roll`

> `fn start_roll(contract_address: ContractAddress, block_number: u64)`

Changes the block number for a contract at the given address.
The change can be canceled with [`stop_roll`](./stop_roll.md).

- `contract_address` - target contract address
- `block_number` - mocked value of block number

For contract implementation:

```rust
// ...
#[external(v0)]
impl IContractImpl of IContract<ContractState> {
    fn get_block_number(ref self: ContractState) -> u64 {
        starknet::get_block_info().unbox().block_number
    }
}
// ...
```

We can use roll in a test to mock block number for a given contract:

```rust
#[test]
fn test_roll() {
    // ...

    start_roll(contract_address, 234);

    let new_block_number = dispatcher.get_block_number().unwrap();
    assert(new_block_number == 234, 'Wrong block number');
}
```

# `stop_roll`

> `fn stop_roll(contract_address: ContractAddress)`

Cancels the [`start_roll`](./start_roll.md) for the contract at the given address.

- `contract_address` - target contract address

```rust
#[test]
fn test_roll() {
    // ...
    
    stop_roll(contract_address);
    
    // ...
}
```


# `start_prank`

> `fn start_prank(target: CheatTarget, caller_address: ContractAddress)`

Changes the caller address for the given target.
This change can be canceled with [`stop_prank`](./stop_prank.md).

- `target` - instance of [`CheatTarget`](./cheat_target.md) specifying which contracts to prank.
- `caller_address` - caller address to be set

For contract implementation:

```rust
// ...
#[storage]
struct Storage {
    // ...
    stored_caller_address: ContractAddress
}

#[abi(embed_v0)]
impl IContractImpl of IContract<ContractState> {
    fn set_caller_address(ref self: ContractState) {
        self.stored_caller_address.write(starknet::get_caller_address());
    }

    fn get_caller_address(self: @ContractState) -> ContractAddress {
        self.stored_caller_address.read()
    }
}
// ...
```

We can use `start_prank` in a test to change the caller address for a given contract:

```rust
use snforge_std::{start_prank, CheatTarget};

#[test]
fn test_prank() {
    // ...

    let caller_address: ContractAddress = 123.try_into().unwrap();

    start_prank(CheatTarget::One(contract_address), caller_address);

    dispatcher.set_caller_address();
    let caller_address = dispatcher.get_caller_address();
    assert(caller_address.into() == 123, 'Wrong caller address');
}
```
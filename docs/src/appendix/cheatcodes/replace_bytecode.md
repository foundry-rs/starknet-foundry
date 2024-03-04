# `replace_bytecode`

> `fn replace_bytecode(contract: ContractAddress, new_class: ClassHash)`

Replaces class for given contract.

- `contract` - address specifying which contracts to cheat on
- `new_class` - class that will be used now for given contract

For contract implementation:

```rust
// ...
#[storage]
struct Storage {
    value: felt252
}

#[abi(embed_v0)]
impl IContractA of super::IContract<ContractState> {
    fn get(self: @ContractState) -> felt252 {
        self.value.read()
    }

    fn set(ref self: ContractState, value: felt252) {
        self.value.write(value);
    }
}
// ...
```
Replacement contract
```rust
// ...
#[storage]
struct Storage {
    value: felt252
}

#[abi(embed_v0)]
impl IContractB of super::IContract<ContractState> {
    fn get(self: @ContractState) -> felt252 {
        self.value.read() + 100
    }

    fn set(ref self: ContractState, value: felt252) {
        self.value.write(value);
    }
}
// ...
```

We can use `replace_bytecode` in a test to change the class hash for contracts:

```rust
use snforge_std::{replace_bytecode, CheatTarget};

#[test]
fn test_replace_bytecode() {
    // ...

    dispatcher.set(50);
    replace_bytecode(contract_address, new_class_hash);

    assert(dispatcher.get() == 150, 'wrong value');
}
```

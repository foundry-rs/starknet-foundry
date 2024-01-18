# `get_class_hash`

> `fn get_class_hash(contract_address: ContractAddress) -> ClassHash`

Retrieves a class hash of a contract deployed under the given address.

- `contract_address` - target contract address

The main purpose of this cheatcode is to test upgradable contracts. For contract implementation:

```rust
// ...
#[abi(embed_v0)]
impl IUpgradeableImpl of super::IUpgradeable<ContractState> {
    fn upgrade(ref self: ContractState, class_hash: starknet::ClassHash) {
        starknet::replace_class_syscall(class_hash).unwrap_syscall();
    }
}
// ...
```

We can use `get_class_hash` to check if it upgraded properly:

```rust
use snforge_std::get_class_hash;

#[test]
fn test_get_class_hash() {
    let contract = declare('Contract1');

    let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();

    assert(get_class_hash(contract_address) == contract.class_hash, 'Incorrect class hash');

    let other_contract = declare('OtherContract');

    IUpgradeableDispatcher { contract_address }.upgrade(other_contract.class_hash);

    assert(get_class_hash(contract_address) == other_contract.class_hash, 'Incorrect class hash upgrade');
}

```

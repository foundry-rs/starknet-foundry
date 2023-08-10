# `get_class_hash`

> `fn get_class_hash(contract_address: ContractAddress) -> ClassHash`

Retrieves the class hash for the given address.

- `contract_address` - target contract address

Let's take a very simple upgradeable contract implementation:

```rust
    use starknet::ClassHash;

    #[starknet::interface]
    trait IUpgradeable<T> {
        fn upgrade(ref self: T, class_hash: ClassHash);
    }

    #[starknet::contract]
    mod Contract1 {
        use result::ResultTrait;

        #[storage]
        struct Storage {
            // ...
        }

        #[external(v0)]
        impl IUpgradeableImpl of super::IUpgradeable<ContractState> {
            fn upgrade(ref self: ContractState, class_hash: starknet::ClassHash) {
                starknet::replace_class_syscall(class_hash).unwrap_syscall();
            }
        }
}
```

The use of this cheatcode can be illustrated as following, where the `replace_class`
syscall will change the class associated to the contract calling it:

```rust
    use snforge_std::{declare, deploy, get_class_hash, PreparedContract};

    #[test]
    fn test_get_class_hash() {
        let class_hash = declare('Contract1');

        let prepared = PreparedContract {
            class_hash: class_hash,
            constructor_calldata: @ArrayTrait::new()
        };

        let contract_address = deploy(prepared).unwrap();

        assert(get_class_hash(contract_address) == class_hash, 'Bad class hash');

        let other_class_hash = declare('OtherContract');

        IUpgradeableDispatcher { contract_address }.upgrade(other_class_hash);

        assert(get_class_hash(contract_address) == other_class_hash, 'Bad class hash upgrade');
    }
```

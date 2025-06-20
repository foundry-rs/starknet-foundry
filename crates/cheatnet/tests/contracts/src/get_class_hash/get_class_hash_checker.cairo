use starknet::ClassHash;

#[starknet::interface]
trait IUpgradeable<T> {
    fn upgrade(ref self: T, class_hash: ClassHash);
}

#[starknet::contract]
mod GetClassHashCheckerUpg {
    use starknet::ClassHash;

    #[storage]
    struct Storage {
        inner: felt252,
    }

    #[abi(embed_v0)]
    impl IUpgradeableImpl of super::IUpgradeable<ContractState> {
        fn upgrade(ref self: ContractState, class_hash: ClassHash) {
            _upgrade(class_hash);
        }
    }

    fn _upgrade(class_hash: ClassHash) {
        match starknet::syscalls::replace_class_syscall(class_hash) {
            Result::Ok(()) => {},
            Result::Err(e) => panic(e),
        };
    }
}

#[starknet::interface]
trait ITraceDummy<T> {
    fn from_proxy_dummy(ref self: T, empty_hash: starknet::ClassHash, salt: felt252);
}

#[starknet::contract]
pub mod TraceDummy {
    pub use starknet::{ClassHash};
    use super::super::use_builtins_and_syscalls;

    #[storage]
    struct Storage {
        balance: u8
    }

    #[abi(embed_v0)]
    pub impl ITraceDummyImpl of super::ITraceDummy<ContractState> {
        fn from_proxy_dummy(ref self: ContractState, empty_hash: ClassHash, salt: felt252) {
            use_builtins_and_syscalls(empty_hash, salt);
        }
    }
}

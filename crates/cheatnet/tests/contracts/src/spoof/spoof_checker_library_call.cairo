use starknet::ClassHash;

#[starknet::interface]
trait ISpoofChecker<TContractState> {
    fn get_tx_hash(ref self: TContractState) -> felt252;
}

#[starknet::interface]
trait ISpoofCheckerLibCall<TContractState> {
    fn get_tx_hash_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> felt252;
}

#[starknet::contract]
mod SpoofCheckerLibCall {
    use super::{ISpoofCheckerDispatcherTrait, ISpoofCheckerLibraryDispatcher};
    use starknet::ClassHash;

    #[storage]
    struct Storage {}

    #[external(v0)]
    impl ISpoofCheckerLibCall of super::ISpoofCheckerLibCall<ContractState> {
        fn get_tx_hash_with_lib_call(ref self: ContractState, class_hash: ClassHash) -> felt252 {
            let spoof_checker = ISpoofCheckerLibraryDispatcher { class_hash };
            spoof_checker.get_tx_hash()
        }
    }
}

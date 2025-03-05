use starknet::ClassHash;

#[starknet::interface]
trait ICheatCallerAddressChecker<TContractState> {
    fn get_caller_address(ref self: TContractState) -> felt252;
}

#[starknet::interface]
trait ICheatCallerAddressCheckerLibCall<TContractState> {
    fn get_caller_address_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> felt252;
    fn get_caller_address(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod CheatCallerAddressCheckerLibCall {
    use super::{
        ICheatCallerAddressCheckerDispatcherTrait, ICheatCallerAddressCheckerLibraryDispatcher,
    };
    use starknet::ClassHash;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatCallerAddressCheckerLibCall of super::ICheatCallerAddressCheckerLibCall<
        ContractState,
    > {
        fn get_caller_address_with_lib_call(
            ref self: ContractState, class_hash: ClassHash,
        ) -> felt252 {
            let cheat_caller_address_checker = ICheatCallerAddressCheckerLibraryDispatcher {
                class_hash,
            };
            cheat_caller_address_checker.get_caller_address()
        }

        fn get_caller_address(ref self: ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }
    }
}

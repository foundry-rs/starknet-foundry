use starknet::ClassHash;

#[starknet::interface]
trait IMockChecker<TContractState> {
    fn get_constant_thing(ref self: TContractState) -> felt252;
}

#[starknet::interface]
trait IMockCheckerLibCall<TContractState> {
    fn get_constant_thing_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> felt252;
}

#[starknet::contract]
mod MockCheckerLibCall {
    use super::{IMockCheckerDispatcherTrait, IMockCheckerLibraryDispatcher};
    use starknet::ClassHash;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IMockCheckerLibCall of super::IMockCheckerLibCall<ContractState> {
        fn get_constant_thing_with_lib_call(
            ref self: ContractState, class_hash: ClassHash
        ) -> felt252 {
            let mock_checker = IMockCheckerLibraryDispatcher { class_hash };
            mock_checker.get_constant_thing()
        }
    }
}

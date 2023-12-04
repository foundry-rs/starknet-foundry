use starknet::ClassHash;

#[starknet::interface]
trait IPrankChecker<TContractState> {
    fn get_caller_address(ref self: TContractState) -> felt252;
}

#[starknet::interface]
trait IPrankCheckerLibCall<TContractState> {
    fn get_caller_address_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> felt252;
}

#[starknet::contract]
mod PrankCheckerLibCall {
    use super::{IPrankCheckerDispatcherTrait, IPrankCheckerLibraryDispatcher};
    use starknet::ClassHash;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IPrankCheckerLibCall of super::IPrankCheckerLibCall<ContractState> {
        fn get_caller_address_with_lib_call(
            ref self: ContractState, class_hash: ClassHash
        ) -> felt252 {
            let prank_checker = IPrankCheckerLibraryDispatcher { class_hash };
            prank_checker.get_caller_address()
        }
    }
}

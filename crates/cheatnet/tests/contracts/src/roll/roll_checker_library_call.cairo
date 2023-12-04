use starknet::ClassHash;

#[starknet::interface]
trait IRollChecker<TContractState> {
    fn get_block_number(ref self: TContractState) -> u64;
}

#[starknet::interface]
trait IRollCheckerLibCall<TContractState> {
    fn get_block_number_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> u64;
}

#[starknet::contract]
mod RollCheckerLibCall {
    use super::{IRollCheckerDispatcherTrait, IRollCheckerLibraryDispatcher};
    use starknet::ClassHash;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IRollCheckerLibCall of super::IRollCheckerLibCall<ContractState> {
        fn get_block_number_with_lib_call(ref self: ContractState, class_hash: ClassHash) -> u64 {
            let roll_checker = IRollCheckerLibraryDispatcher { class_hash };
            roll_checker.get_block_number()
        }
    }
}

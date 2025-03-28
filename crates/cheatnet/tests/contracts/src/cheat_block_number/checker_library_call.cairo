use starknet::ClassHash;

#[starknet::interface]
trait ICheatBlockNumberChecker<TContractState> {
    fn get_block_number(ref self: TContractState) -> u64;
}

#[starknet::interface]
trait ICheatBlockNumberCheckerLibCall<TContractState> {
    fn get_block_number_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> u64;
    fn get_block_number(ref self: TContractState) -> u64;
}

#[starknet::contract]
mod CheatBlockNumberCheckerLibCall {
    use super::{ICheatBlockNumberCheckerDispatcherTrait, ICheatBlockNumberCheckerLibraryDispatcher};
    use starknet::ClassHash;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockNumberCheckerLibCall of super::ICheatBlockNumberCheckerLibCall<ContractState> {
        fn get_block_number_with_lib_call(ref self: ContractState, class_hash: ClassHash) -> u64 {
            let cheat_block_number_checker = ICheatBlockNumberCheckerLibraryDispatcher {
                class_hash
            };
            cheat_block_number_checker.get_block_number()
        }

        fn get_block_number(ref self: ContractState) -> u64 {
            starknet::get_block_info().unbox().block_number
        }
    }
}

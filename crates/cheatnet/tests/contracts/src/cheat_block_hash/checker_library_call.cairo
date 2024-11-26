use starknet::ClassHash;

#[starknet::interface]
trait ICheatBlockHashChecker<TContractState> {
    fn get_block_hash(ref self: TContractState) -> felt252;
}

#[starknet::interface]
trait ICheatBlockHashCheckerLibCall<TContractState> {
    fn get_block_hash_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> felt252;
    fn get_block_hash(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod CheatBlockHashCheckerLibCall {
    use super::{ICheatBlockHashCheckerDispatcherTrait, ICheatBlockHashCheckerLibraryDispatcher};
    use core::starknet::SyscallResultTrait;
    use starknet::ClassHash;
    use starknet::{get_block_info, get_block_hash_syscall};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockHashCheckerLibCall of super::ICheatBlockHashCheckerLibCall<ContractState> {
        fn get_block_hash_with_lib_call(ref self: ContractState, class_hash: ClassHash) -> felt252 {
            let cheat_block_hash_checker = ICheatBlockHashCheckerLibraryDispatcher { class_hash };
            cheat_block_hash_checker.get_block_hash()
        }

        fn get_block_hash(ref self: ContractState) -> felt252 {
            let block_info = get_block_info().unbox();
            let block_hash = get_block_hash_syscall(block_info.block_number - 10).unwrap_syscall();

            block_hash
        }
    }
}

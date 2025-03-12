use starknet::ClassHash;

#[starknet::interface]
trait ICheatBlockHashChecker<TContractState> {
    fn get_block_hash(ref self: TContractState, block_number: u64) -> felt252;
}

#[starknet::interface]
trait ICheatBlockHashCheckerLibCall<TContractState> {
    fn get_block_hash_with_lib_call(ref self: TContractState, class_hash: ClassHash, block_number: u64) -> felt252;
    fn get_block_hash(ref self: TContractState, block_number: u64) -> felt252;
}

#[starknet::contract]
mod CheatBlockHashCheckerLibCall {
    use super::{ICheatBlockHashCheckerDispatcherTrait, ICheatBlockHashCheckerLibraryDispatcher};
    use core::starknet::SyscallResultTrait;
    use starknet::ClassHash;
    use starknet::syscalls::get_block_hash_syscall;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockHashCheckerLibCall of super::ICheatBlockHashCheckerLibCall<ContractState> {
        fn get_block_hash_with_lib_call(ref self: ContractState, class_hash: ClassHash, block_number: u64) -> felt252 {
            let cheat_block_hash_checker = ICheatBlockHashCheckerLibraryDispatcher { class_hash };
            cheat_block_hash_checker.get_block_hash(block_number)
        }

        fn get_block_hash(ref self: ContractState, block_number: u64) -> felt252 {
            let block_hash = get_block_hash_syscall(block_number).unwrap_syscall();

            block_hash
        }
    }
}

#[starknet::interface]
trait ICheatBlockHashChecker<TContractState> {
    fn get_block_hash(ref self: TContractState) -> felt252;
    fn get_block_hash_and_number(ref self: TContractState) -> (felt252, u64);
}

#[starknet::contract]
mod CheatBlockHashChecker {
    use core::starknet::SyscallResultTrait;
    use starknet::{get_block_info, syscalls::get_block_hash_syscall};

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        BlockHashEmitted: BlockHashEmitted
    }

    #[derive(Drop, starknet::Event)]
    struct BlockHashEmitted {
        block_hash: felt252
    }


    #[abi(embed_v0)]
    impl CheatBlockHashChecker of super::ICheatBlockHashChecker<ContractState> {
        fn get_block_hash(ref self: ContractState) -> felt252 {
            let block_info = get_block_info().unbox();
            let block_number = block_info.block_number - 10;
            let block_hash = get_block_hash_syscall(block_number).unwrap_syscall();

            block_hash
        }

        fn get_block_hash_and_number(ref self: ContractState) -> (felt252, u64) {
            let block_info = starknet::get_block_info().unbox();
            let block_number = block_info.block_number - 10;
            let block_hash = get_block_hash_syscall(block_number).unwrap_syscall();

            (block_hash, block_number)
        }
    }
}

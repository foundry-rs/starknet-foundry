#[starknet::interface]
trait ICheatBlockHashChecker<TContractState> {
    fn get_block_hash(ref self: TContractState, block_number: u64) -> felt252;
}

#[starknet::contract]
mod CheatBlockHashChecker {
    use core::starknet::SyscallResultTrait;
    use starknet::syscalls::get_block_hash_syscall;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl CheatBlockHashChecker of super::ICheatBlockHashChecker<ContractState> {
        fn get_block_hash(ref self: ContractState, block_number: u64) -> felt252 {
            let block_hash = get_block_hash_syscall(block_number).unwrap_syscall();

            block_hash
        }
    }
}

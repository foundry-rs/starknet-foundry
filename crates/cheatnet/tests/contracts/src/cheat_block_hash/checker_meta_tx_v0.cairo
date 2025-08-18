#[starknet::interface]
trait ICheatBlockHashCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState, block_number: u64) -> felt252;
    fn __validate__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatBlockHashCheckerMetaTxV0 {
    use starknet::SyscallResultTrait;
    use starknet::syscalls::get_block_hash_syscall;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockHashCheckerMetaTxV0 of super::ICheatBlockHashCheckerMetaTxV0<ContractState> {
        fn __execute__(ref self: ContractState, block_number: u64) -> felt252 {
            let block_hash = get_block_hash_syscall(block_number).unwrap_syscall();

            block_hash
        }

        fn __validate__(ref self: ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }
}

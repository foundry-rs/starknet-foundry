#[starknet::interface]
trait IReplaceInFork<TContractState> {
    fn get_admin(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod ReplaceInFork {
    use starknet::{SyscallResultTrait, SyscallResult, syscalls::get_execution_info_v2_syscall};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IReplaceInFork of super::IReplaceInFork<ContractState> {
        fn get_admin(self: @ContractState) -> felt252 {
            0
        }
    }
}

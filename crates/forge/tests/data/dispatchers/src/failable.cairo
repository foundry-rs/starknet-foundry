#[starknet::interface]
pub trait IFailableContract<TContractState> {
    fn recoverable_panic(self: @TContractState);
    fn unrecoverable_error(self: @TContractState);
}

#[starknet::contract]
pub mod FailableContract {
    use starknet::SyscallResultTrait;
    #[storage]
    pub struct Storage {}

    #[abi(embed_v0)]
    impl FailableContract of super::IFailableContract<ContractState> {
        fn recoverable_panic(self: @ContractState) {
            core::panic_with_felt252('Errare humanum est');
        }

        fn unrecoverable_error(self: @ContractState) {
            // Call syscall with nonexistent address should fail immediately
            starknet::syscalls::call_contract_syscall(
                0x123.try_into().unwrap(), 0x1, array![].span(),
            )
                .unwrap_syscall();
        }
    }
}

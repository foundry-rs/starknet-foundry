#[starknet::interface]
pub trait ISpySyscallEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
    fn emit_event_with_syscall(ref self: TContractState, some_key: felt252, some_data: felt252);
}

#[starknet::contract]
pub mod SpySyscallEventsChecker {
    // ...
    // Rest of the implementation identical to `SpyEventsChecker`

    use core::starknet::{SyscallResultTrait, syscalls::emit_event_syscall};

    #[external(v0)]
    pub fn emit_event_with_syscall(ref self: ContractState, some_key: felt252, some_data: felt252) {
        emit_event_syscall(array![some_key].span(), array![some_data].span()).unwrap_syscall();
    }
}

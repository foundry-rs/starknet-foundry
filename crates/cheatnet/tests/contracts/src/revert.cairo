#[starknet::interface]
pub trait IRevert<TContractState> {
    fn call_contract_revert(
        ref self: TContractState,
        contract_address: starknet::ContractAddress,
        entry_point_selector: felt252,
        new_class_hash: starknet::ClassHash,
    );
    fn change_state_and_panic(ref self: TContractState, class_hash: starknet::ClassHash);
}

#[starknet::contract]
mod Revert {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use starknet::{SyscallResultTrait, syscalls};

    #[storage]
    struct Storage {
        storage_var: felt252,
    }


    #[abi(embed_v0)]
    impl RevertImpl of super::IRevert<ContractState> {
        fn call_contract_revert(
            ref self: ContractState,
            contract_address: starknet::ContractAddress,
            entry_point_selector: felt252,
            new_class_hash: starknet::ClassHash,
        ) {
            match syscalls::call_contract_syscall(
                contract_address, entry_point_selector, array![new_class_hash.into()].span(),
            ) {
                Result::Ok(_) => core::panic_with_felt252('expected revert'),
                Result::Err(errors) => {
                    let mut error_span = errors.span();
                    assert(
                        *error_span.pop_back().unwrap() == 'ENTRYPOINT_FAILED', 'unexpected error',
                    );
                },
            }
            assert(self.storage_var.read() == 0, 'values should not change');
        }

        fn change_state_and_panic(ref self: ContractState, class_hash: starknet::ClassHash) {
            let dummy_span = array![0].span();
            syscalls::emit_event_syscall(dummy_span, dummy_span).unwrap_syscall();
            syscalls::replace_class_syscall(class_hash).unwrap_syscall();
            syscalls::send_message_to_l1_syscall(17.try_into().unwrap(), dummy_span)
                .unwrap_syscall();

            self.storage_var.write(987);
            assert(self.storage_var.read() == 987, 'values should change');

            core::panic_with_felt252('change_state_and_panic');
        }
    }
}

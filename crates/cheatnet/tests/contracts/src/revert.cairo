#[starknet::contract]
mod Revert {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use starknet::{StorageAddress, SyscallResultTrait, syscalls};

    #[storage]
    struct Storage {
        storage_var: felt252,
    }

    #[external(v0)]
    fn modify_in_nested_call_and_handle_panic(
        ref self: ContractState,
        contract_address: starknet::ContractAddress,
        new_class_hash: starknet::ClassHash,
    ) {
        match syscalls::call_contract_syscall(
            contract_address,
            selector!("modify_contract_var_and_panic"),
            array![new_class_hash.into()].span(),
        ) {
            Result::Ok(_) => core::panic_with_felt252('expected revert'),
            Result::Err(errors) => {
                let mut error_span = errors.span();
                assert(*error_span.pop_back().unwrap() == 'ENTRYPOINT_FAILED', 'unexpected error');
            },
        }
        assert(self.storage_var.read() == 0, 'value should not change');
    }

    #[external(v0)]
    fn modify_in_top_and_nested_calls_and_panic(ref self: ContractState, key: StorageAddress) {
        let storage_before = syscalls::storage_read_syscall(0, key).unwrap_syscall();
        assert(storage_before == 0, 'incorrect storage before');

        // Call `modify_specific_storage` without panic.
        syscalls::call_contract_syscall(
            starknet::get_contract_address(),
            selector!("modify_specific_storage"),
            array![key.into(), 99, 0].span(),
        )
            .unwrap_syscall();

        let storage_after = syscalls::storage_read_syscall(0, key).unwrap_syscall();
        assert(storage_after == 99, 'incorrect storage after');

        let dummy_span = array![1, 1].span();
        syscalls::emit_event_syscall(dummy_span, dummy_span).unwrap_syscall();
        syscalls::send_message_to_l1_syscall(91, dummy_span).unwrap_syscall();

        // Call `modify_specific_storage` with panic.
        syscalls::call_contract_syscall(
            starknet::get_contract_address(),
            selector!("modify_specific_storage"),
            array![key.into(), 88, 1].span(),
        )
            .unwrap_syscall();

        assert(false, 'unreachable');
    }

    #[external(v0)]
    fn modify_contract_var_and_panic(ref self: ContractState, class_hash: starknet::ClassHash) {
        let dummy_span = array![0].span();
        syscalls::emit_event_syscall(dummy_span, dummy_span).unwrap_syscall();
        syscalls::replace_class_syscall(class_hash).unwrap_syscall();
        syscalls::send_message_to_l1_syscall(17, dummy_span).unwrap_syscall();

        self.storage_var.write(987);
        assert(self.storage_var.read() == 987, 'value should change');

        core::panic_with_felt252('modify_contract_var_and_panic');
    }

    #[external(v0)]
    fn modify_specific_storage(
        ref self: ContractState, key: StorageAddress, new_value: felt252, should_panic: bool,
    ) {
        let address_domain = 0;
        syscalls::storage_write_syscall(address_domain, key, new_value).unwrap_syscall();

        let dummy_span = array![0].span();
        syscalls::emit_event_syscall(dummy_span, dummy_span).unwrap_syscall();
        syscalls::send_message_to_l1_syscall(19, dummy_span).unwrap_syscall();

        if should_panic {
            core::panic_with_felt252('modify_specific_storage');
        }
    }
}

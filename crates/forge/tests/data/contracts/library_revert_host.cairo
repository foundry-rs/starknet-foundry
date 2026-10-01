#[starknet::contract]
mod LibraryRevertHost {
    use starknet::ClassHash;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        value: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        self.value.write(7);
    }

    #[external(v0)]
    fn attempt(ref self: ContractState, library: ClassHash) -> bool {
        starknet::syscalls::library_call_syscall(
            library, selector!("write_storage_and_panic"), array![11].span(),
        ).is_ok()
    }

    #[external(v0)]
    fn host_value(self: @ContractState) -> felt252 {
        self.value.read()
    }
}

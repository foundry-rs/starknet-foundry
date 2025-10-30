#[starknet::interface]
trait IContract<TContractState> {
    /// Write 1 to storage storage and then panic
    fn call_with_unrecoverable(ref self: TContractState);
    /// Return storage value
    fn read_storage(self: @TContractState) -> felt252;
    /// Write 5 to storage
    fn write_storage(ref self: TContractState);
}

#[starknet::contract]
mod Contract {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use starknet::syscalls::call_contract_syscall;
    use starknet::syscalls::deploy_syscall;

    #[storage]
    struct Storage {
        value: felt252,
    }

    #[abi(embed_v0)]
    impl IContractImpl of super::IContract<ContractState> {
        fn call_with_unrecoverable(ref self: ContractState) {
            self.value.write(1);
            // Call non-existing contract
            // call_contract_syscall(0x123456.try_into().unwrap(), 0, array![].span()).unwrap();
            deploy_syscall(0x123456.try_into().unwrap(), 0, array![].span(), false).unwrap();
        }

        fn read_storage(self: @ContractState) -> felt252 {
            self.value.read()
        }

        fn write_storage(ref self: ContractState) {
            self.value.write(5);
        }
    }
}

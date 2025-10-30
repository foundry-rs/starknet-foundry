use starknet::SyscallResultTrait;

#[starknet::interface]
/// Makes calls to nested contract with safe dispatcher
trait ISafeProxy<TContractState> {
    /// Call on proxied contract with safe dispatcher: Write 1 to storage and then panic
    fn call_write_storage_and_handle_panic(ref self: TContractState);
    /// Call on proxied contract unwraping the syscall result: Return storage value
    fn read_storage(self: @TContractState) -> felt252;
    /// Call on proxied contract unwraping the syscall result: Write 5 to storage
    fn write_storage(ref self: TContractState);
}

#[starknet::interface]
trait IContract<TContractState> {
    /// Write `value` to storage and then panic
    fn write_storage_and_panic(ref self: TContractState, value: felt252);
    /// Return storage value
    fn read_storage(self: @TContractState) -> felt252;
    /// Write `value` to storage and emits event
    fn write_storage(ref self: TContractState, value: felt252);
}

#[starknet::contract]
#[feature("safe_dispatcher")]
mod SafeProxy {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use starknet::{ContractAddress, SyscallResultTrait};
    use super::{IContractSafeDispatcher, IContractSafeDispatcherTrait, ISafeProxy};

    #[storage]
    struct Storage {
        address: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, address: ContractAddress) {
        self.address.write(address);
    }

    #[abi(embed_v0)]
    impl ISafeProxyImpl of ISafeProxy<ContractState> {
        fn call_write_storage_and_handle_panic(ref self: ContractState) {
            let contract_address = self.address.read();
            let dispatcher = IContractSafeDispatcher { contract_address };
            let storage = dispatcher.read_storage().unwrap_syscall();
            assert(storage == 5, 'Storage should be 5');

            if let Ok(_) = dispatcher.write_storage_and_panic(1) {
                panic!("Should have panicked")
            }
        }

        fn read_storage(self: @ContractState) -> felt252 {
            let contract_address = self.address.read();
            let dispatcher = IContractSafeDispatcher { contract_address };
            dispatcher.read_storage().unwrap_syscall()
        }

        fn write_storage(ref self: ContractState) {
            let contract_address = self.address.read();
            let dispatcher = IContractSafeDispatcher { contract_address };
            dispatcher.write_storage(5).unwrap_syscall()
        }
    }
}

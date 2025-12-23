use starknet::SyscallResultTrait;

#[starknet::interface]
/// Makes calls to nested contract with safe dispatcher
trait ISafeProxy<TContractState> {
    /// Call on proxied contract with safe dispatcher: Write 1 to storage storage and then panic
    fn call_with_panic(ref self: TContractState);
    /// Call on proxied contract unwraping the syscall result: Return storage value
    fn read_storage(self: @TContractState) -> felt252;
    /// Call on proxied contract unwraping the syscall result: Write 5 to storage
    fn write_storage(ref self: TContractState);
}

#[starknet::interface]
trait IContract<TContractState> {
    /// Write 1 to storage storage and then panic
    fn call_with_panic(ref self: TContractState);
    /// Return storage value
    fn read_storage(self: @TContractState) -> felt252;
    /// Write 5 to storage
    fn write_storage(ref self: TContractState);
}

#[starknet::contract]
mod SafeProxy {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use starknet::{ContractAddress, SyscallResultTrait};
    use super::{IContractSafeDispatcher, IContractSafeDispatcherTrait, ISafeProxy};

    #[storage]
    struct Storage {
        addr: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, addr: ContractAddress) {
        self.addr.write(addr);
    }

    #[abi(embed_v0)]
    impl ISafeProxyImpl of ISafeProxy<ContractState> {
        fn call_with_panic(ref self: ContractState) {
            let contract_address = self.addr.read();
            let dispatcher = IContractSafeDispatcher { contract_address };
            let storage = dispatcher.read_storage().unwrap_syscall();
            assert(storage == 5, 'storage should be 5');
            match dispatcher.call_with_panic() {
                Ok(_) => panic!("Should have panicked"),
                Err(_) => {// Handle panic
                },
            }
            assert(false, 'should not execute');
        }

        fn read_storage(self: @ContractState) -> felt252 {
            let contract_address = self.addr.read();
            let dispatcher = IContractSafeDispatcher { contract_address };
            dispatcher.read_storage().unwrap_syscall()
        }

        fn write_storage(ref self: ContractState) {
            let contract_address = self.addr.read();
            let dispatcher = IContractSafeDispatcher { contract_address };
            dispatcher.write_storage().unwrap_syscall()
        }
    }
}

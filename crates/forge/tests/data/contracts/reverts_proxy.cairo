use starknet::ContractAddress;

#[starknet::interface]
trait IContract<TContractState> {
    /// Return storage value
    fn read_storage(self: @TContractState) -> felt252;
    /// Write `value` to storage and emits event
    fn write_storage(ref self: TContractState, value: felt252);
    /// Write `value` to storage and then panic
    fn write_storage_and_panic(ref self: TContractState, value: felt252);
}

#[starknet::interface]
/// Makes calls to nested contract with address `address` 
trait ILibraryProxy<TContractState> {
    /// Call on proxied contract unwrapping the syscall result: Return storage value
    fn library_read_storage(self: @TContractState, address: ContractAddress) -> felt252;
    /// Call on proxied contract unwrapping the syscall result: Write `value` to storage
    fn library_write_storage(self: @TContractState, address: ContractAddress, value: felt252);
    /// Call on proxied contract with safe dispatcher: Write `value` to storage and then handle panic
    fn library_write_storage_and_panic(self: @TContractState, address: ContractAddress, value: felt252);
}

#[starknet::interface]
/// Makes calls to nested contract with safe dispatcher
trait ISafeProxy<TContractState> {
    /// Call on proxied contract with safe dispatcher: Write `value` to storage and then handle panic
    fn call_write_storage_and_handle_panic(ref self: TContractState, value: felt252);
}

#[feature("safe_dispatcher")]
#[starknet::contract]
mod Proxy {
    use starknet::{ContractAddress, SyscallResultTrait};
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use super::{IContract, ISafeProxy, ILibraryProxy, IContractDispatcher, IContractSafeDispatcher, IContractDispatcherTrait, IContractSafeDispatcherTrait};

    #[storage]
    struct Storage {
        address: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, address: ContractAddress) {
        self.address.write(address);
    }

    #[abi(embed_v0)]
    impl IProxyImpl of IContract<ContractState> {
        fn write_storage_and_panic(ref self: ContractState, value: felt252) {
            let contract_address = self.address.read();
            let dispatcher = IContractDispatcher { contract_address };
            let storage = dispatcher.read_storage();
            assert(storage != 0, 'Storage already modified');

            dispatcher.write_storage(43);
            let storage = dispatcher.read_storage();
            assert(storage == 43, 'Incorrect storage value');

            dispatcher.write_storage_and_panic(value);

            // unreachable
            assert(false, 'Should not execute');
        }

        fn read_storage(self: @ContractState) -> felt252 {
            let contract_address = self.address.read();
            let dispatcher = IContractDispatcher { contract_address };
            dispatcher.read_storage()
        }

        fn write_storage(ref self: ContractState, value: felt252) {
            let contract_address = self.address.read();
            let dispatcher = IContractDispatcher { contract_address };
            dispatcher.write_storage(value)
        }
    }

    #[abi(embed_v0)]
    impl ILibraryProxyImpl of ILibraryProxy<ContractState> {
        fn library_write_storage_and_panic(self: @ContractState, address: ContractAddress, value: felt252) {
            let dispatcher = IContractDispatcher { contract_address: address };
            let storage = dispatcher.read_storage();
            assert(storage != 0, 'Storage already modified');

            dispatcher.write_storage(43);
            let storage = dispatcher.read_storage();
            assert(storage == 43, 'Incorrect storage value');

            dispatcher.write_storage_and_panic(value);

            // unreachable
            assert(false, 'Should not execute');
        }

        fn library_read_storage(self: @ContractState, address: ContractAddress) -> felt252 {
            let dispatcher = IContractDispatcher { contract_address: address };
            dispatcher.read_storage()
        }

        fn library_write_storage(self: @ContractState, address: ContractAddress, value: felt252) {
            let dispatcher = IContractDispatcher { contract_address: address };
            dispatcher.write_storage(value)
        }
    }

    #[abi(embed_v0)]
    impl ISafeProxyImpl of ISafeProxy<ContractState> {
        fn call_write_storage_and_handle_panic(ref self: ContractState, value: felt252) {
            let contract_address = self.address.read();
            let dispatcher = IContractSafeDispatcher { contract_address };
            let storage = dispatcher.read_storage().unwrap_syscall();
            assert(storage != 0, 'Storage already modified');

            if let Ok(_) = dispatcher.write_storage_and_panic(value) {
                panic!("Should have panicked")
            }
        }
    }
}

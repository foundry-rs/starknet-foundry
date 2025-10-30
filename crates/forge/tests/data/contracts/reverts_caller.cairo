use starknet::SyscallResultTrait;

#[starknet::interface]
trait ICaller<TContractState> {
    /// Execute test scenario in tests
    fn call(ref self: TContractState);
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
mod Caller {
    use starknet::ContractAddress;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use super::{ICaller, IContractSafeDispatcher, IContractSafeDispatcherTrait};

    #[storage]
    struct Storage {
        address: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, address: ContractAddress) {
        self.address.write(address);
    }

    #[abi(embed_v0)]
    impl ICallerImpl of ICaller<ContractState> {
        fn call(ref self: ContractState) {
            let contract_address = self.addr.read();
            let dispatcher = IContractSafeDispatcher { contract_address };

            dispatcher.write_storage(43).unwrap();

            // Make sure the storage is updated
            let storage = dispatcher.read_storage().unwrap();
            assert(storage == 43, 'Incorrect storage');

            // Try modifying storage and handle panic
            match dispatcher.write_storage_and_panic(1) {
                Result::Ok(_) => panic!("Should have panicked"),
                Result::Err(_) => { // handled
                },
            }

            // Check storage change was reverted
            let storage = dispatcher.read_storage().unwrap();
            assert(storage == 43, 'Storage not reverted');
        }
    }
}

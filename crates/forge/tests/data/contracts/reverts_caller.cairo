use starknet::SyscallResultTrait;

#[starknet::interface]
trait ICaller<TContractState> {
    /// Execute test scenario in tests
    fn call(ref self: TContractState);
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
mod Caller {
    use starknet::ContractAddress;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use super::{ICaller, IContractSafeDispatcher, IContractSafeDispatcherTrait};

    #[storage]
    struct Storage {
        addr: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, addr: ContractAddress) {
        self.addr.write(addr);
    }

    #[abi(embed_v0)]
    impl ICallerImpl of ICaller<ContractState> {
        fn call(ref self: ContractState) {
            let contract_address = self.addr.read();
            let dispatcher = IContractSafeDispatcher { contract_address };

            // Write 5 to storage
            dispatcher.write_storage().unwrap();

            // Check written value value
            let storage = dispatcher.read_storage().unwrap();
            assert(storage == 5, 'storage not 5');

            // Try modifying storage and handle panic
            match dispatcher.call_with_panic() {
                Result::Ok(_) => panic!("Should have panicked"),
                Result::Err(_panic_data) => { // handled
                },
            }

            // Check storage change was reverted
            let storage = dispatcher.read_storage().unwrap();
            assert(storage == 5, 'storage not reverted');
        }
    }
}

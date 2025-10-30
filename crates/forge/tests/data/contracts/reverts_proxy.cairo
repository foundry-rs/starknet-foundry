#[starknet::interface]
trait IProxy<TContractState> {
    /// Call on proxied contract: Write 1 to storage and then panic
    fn write_storage_and_panic(ref self: TContractState);
    /// Call on proxied contract: Return storage value
    fn read_storage(self: @TContractState) -> felt252;
    /// Call on proxied contract: Write 5 to storage
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
mod Proxy {
    use starknet::ContractAddress;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use super::{IContractDispatcher, IContractDispatcherTrait, IProxy};

    #[storage]
    struct Storage {
        address: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, address: ContractAddress) {
        self.address.write(address);
    }

    #[abi(embed_v0)]
    impl IProxyImpl of IProxy<ContractState> {
        fn write_storage_and_panic(ref self: ContractState) {
            let contract_address = self.addr.read();
            let dispatcher = IContractDispatcher { contract_address };
            let storage = dispatcher.read_storage();
            assert(storage == 0, 'Storage should be 0');
            dispatcher.write_storage_and_panic(1);

            // unreachable
            assert(false, 'Should not execute');
        }

        fn read_storage(self: @ContractState) -> felt252 {
            let contract_address = self.address.read();
            let dispatcher = IContractDispatcher { contract_address };
            dispatcher.read_storage()
        }

        fn write_storage(ref self: ContractState) {
            let contract_address = self.address.read();
            let dispatcher = IContractDispatcher { contract_address };
            dispatcher.write_storage(5)
        }
    }
}

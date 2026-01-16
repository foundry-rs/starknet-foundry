#[starknet::interface]
trait IProxy<TContractState> {
    /// Call on proxied contract: Write 1 to storage storage and then panic
    fn call_with_panic(ref self: TContractState);
    /// Call on proxied contract: Return storage value
    fn read_storage(self: @TContractState) -> felt252;
    /// Call on proxied contract: Write 5 to storage
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
mod Proxy {
    use starknet::ContractAddress;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use super::{IContractDispatcher, IContractDispatcherTrait, IProxy};

    #[storage]
    struct Storage {
        addr: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, addr: ContractAddress) {
        self.addr.write(addr);
    }

    #[abi(embed_v0)]
    impl IProxyImpl of IProxy<ContractState> {
        fn call_with_panic(ref self: ContractState) {
            let contract_address = self.addr.read();
            let dispatcher = IContractDispatcher { contract_address };
            let storage = dispatcher.read_storage();
            assert(storage == 0, 'storage should be 0');
            dispatcher.call_with_panic();
            assert(false, 'should not execute');
        }

        fn read_storage(self: @ContractState) -> felt252 {
            let contract_address = self.addr.read();
            let dispatcher = IContractDispatcher { contract_address };
            dispatcher.read_storage()
        }

        fn write_storage(ref self: ContractState) {
            let contract_address = self.addr.read();
            let dispatcher = IContractDispatcher { contract_address };
            dispatcher.write_storage()
        }
    }
}

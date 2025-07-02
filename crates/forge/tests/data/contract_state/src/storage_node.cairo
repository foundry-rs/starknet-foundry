#[starknet::interface]
pub trait IStorageNodeContract<TContractState> {
    fn get_description_at(self: @TContractState, index: u64) -> felt252;
    fn get_data_at(
        self: @TContractState, index: u64, address: starknet::ContractAddress, key: u16,
    ) -> ByteArray;
}

#[starknet::contract]
pub mod StorageNodeContract {
    use starknet::ContractAddress;
    use starknet::storage::{StoragePointerReadAccess, StoragePathEntry, Map};

    #[starknet::storage_node]
    pub struct RandomData {
        pub description: felt252,
        pub data: Map<(ContractAddress, u16), ByteArray>,
    }

    #[storage]
    pub struct Storage {
        pub random_data: Map<u64, RandomData>,
    }

    #[abi(embed_v0)]
    impl IStorageNodeContractImpl of super::IStorageNodeContract<ContractState> {
        fn get_description_at(self: @ContractState, index: u64) -> felt252 {
            self.random_data.entry(index).description.read()
        }

        fn get_data_at(
            self: @ContractState, index: u64, address: ContractAddress, key: u16,
        ) -> ByteArray {
            self.random_data.entry(index).data.entry((address, key)).read()
        }
    }
}

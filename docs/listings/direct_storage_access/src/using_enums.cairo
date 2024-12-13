#[starknet::interface]
pub trait IEnumsStorageContract<TContractState> {
    fn write_value(ref self: TContractState, key: u256, value: Option<u256>);
    fn read_value(self: @TContractState, key: u256) -> Option<u256>;
}

#[starknet::contract]
pub mod EnumsStorageContract {
    use starknet::{storage::{StoragePointerWriteAccess, StoragePathEntry, Map}};

    #[storage]
    struct Storage {
        example_storage: Map<u256, Option<u256>>,
    }

    #[abi(embed_v0)]
    impl EnumsStorageContractImpl of super::IEnumsStorageContract<ContractState> {
        fn write_value(ref self: ContractState, key: u256, value: Option<u256>) {
            self.example_storage.entry(key).write(value);
        }

        fn read_value(self: @ContractState, key: u256) -> Option<u256> {
            self.example_storage.entry(key).read()
        }
    }
}

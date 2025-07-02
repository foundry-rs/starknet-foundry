#[starknet::contract]
pub mod StorageNodeContract {
    use starknet::ContractAddress;
    use starknet::storage::Map;

    #[starknet::storage_node]
    struct RandomData {
        title: felt252,
        description: felt252,
        counter: u32,
        data: Map<(ContractAddress, u16), ByteArray>,
    }

    #[storage]
    struct Storage {
        pub random_data: RandomData,
    }
}

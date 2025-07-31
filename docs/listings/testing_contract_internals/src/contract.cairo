#[starknet::interface]
pub trait IContract<TContractState> {
    fn get_balance_at(self: @TContractState, address: starknet::ContractAddress) -> u64;
}

#[starknet::contract]
pub mod Contract {
    use starknet::storage::{
        StoragePointerReadAccess, StorageMapReadAccess, StorageMapWriteAccess, Map,
    };
    use starknet::ContractAddress;

    #[storage]
    pub struct Storage {
        pub balances: Map<ContractAddress, u64>,
    }

    #[abi(embed_v0)]
    impl ContractImpl of super::IContract<ContractState> {
        fn get_balance_at(self: @ContractState, address: ContractAddress) -> u64 {
            self.balances.read(address)
        }
    }

    #[generate_trait]
    pub impl InternalImpl of InternalTrait {
        fn _internal_set_balance(ref self: ContractState, address: ContractAddress, balance: u64) {
            self.balances.write(address, balance);
        }
    }
}

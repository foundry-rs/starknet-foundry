#[starknet::interface]
pub trait ISimpleContract<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
}

#[starknet::contract]
pub mod SimpleContract {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[abi(embed_v0)]
    pub impl SimpleContractImpl of super::ISimpleContract<ContractState> {
        // Increases the balance by the given amount
        fn increase_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(self.balance.read() + amount);
        }

        // Gets the balance.
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
    }
}

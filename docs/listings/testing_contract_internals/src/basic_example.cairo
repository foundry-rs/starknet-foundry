#[starknet::interface]
trait IContract<TContractState> {}

#[starknet::contract]
pub mod Contract {
    use starknet::storage::{StoragePointerReadAccess};
    #[storage]
    pub struct Storage {
        pub balance: felt252,
    }

    #[generate_trait]
    pub impl InternalImpl of InternalTrait {
        fn internal_function(self: @ContractState) -> felt252 {
            self.balance.read()
        }
    }

    pub fn other_internal_function(self: @ContractState) -> felt252 {
        self.balance.read() + 5
    }
}

#[cfg(test)]
mod tests {
    use core::starknet::storage::{
        StoragePointerReadAccess, StoragePointerWriteAccess,
    }; // <--- Ad. 1
    use super::Contract;
    use super::Contract::{InternalTrait, other_internal_function}; // <--- Ad. 2

    #[test]
    fn test_internal() {
        let mut state = Contract::contract_state_for_testing(); // <--- Ad. 3
        state.balance.write(10);

        let value = state.internal_function();
        assert(value == 10, 'Incorrect storage value');

        let other_value = other_internal_function(@state);
        assert(other_value == 15, 'Incorrect return value');
    }
}

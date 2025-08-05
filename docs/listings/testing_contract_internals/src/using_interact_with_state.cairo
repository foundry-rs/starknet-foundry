#[starknet::interface]
trait IContract<TContractState> {
    fn get_balance(self: @TContractState, address: starknet::ContractAddress) -> u64;
}

#[starknet::contract]
pub mod Contract {
    use starknet::ContractAddress;
    use starknet::storage::{Map, StorageMapReadAccess, StorageMapWriteAccess};

    #[storage]
    pub struct Storage {
        pub balances: Map<ContractAddress, u64>,
    }

    #[abi(embed_v0)]
    impl ContractImpl of super::IContract<ContractState> {
        fn get_balance(self: @ContractState, address: ContractAddress) -> u64 {
            self.balances.read(address)
        }
    }

    #[generate_trait]
    pub impl InternalImpl of InternalTrait {
        fn _set_balance(ref self: ContractState, address: ContractAddress, balance: u64) {
            self.balances.write(address, balance);
        }
    }
}

#[cfg(test)]
mod tests {
    // 0. Import necessary structs and traits
    use snforge_std::{ContractClassTrait, DeclareResultTrait, declare, interact_with_state};
    use starknet::ContractAddress;
    use starknet::storage::{StorageMapReadAccess, StorageMapWriteAccess};
    use super::Contract::InternalTrait;
    use super::{Contract, IContractDispatcher, IContractDispatcherTrait};

    fn deploy_contract() -> starknet::ContractAddress {
        let contract = declare("Contract").unwrap().contract_class();
        let (contract_address, _) = contract.deploy(@array![]).unwrap();
        contract_address
    }

    #[test]
    fn test_storage() {
        // 1. Deploy your contract
        let contract_address = deploy_contract();
        let dispatcher = IContractDispatcher { contract_address };

        let contract_to_modify: ContractAddress = 0x123.try_into().unwrap();

        assert(dispatcher.get_balance(contract_to_modify) == 0, 'Wrong balance');

        // 2. Use `interact_with_state` to access and modify the contract's storage
        interact_with_state(
            contract_address,
            || {
                // 3. Get access to the contract's state
                let mut state = Contract::contract_state_for_testing();

                // 4. Read from storage
                let current_balance = state.balances.read(contract_to_modify);

                // 5. Write to storage
                state.balances.write(contract_to_modify, current_balance + 100);
            },
        );

        assert(dispatcher.get_balance(contract_to_modify) == 100, 'Wrong balance');
    }

    #[test]
    fn test_internal_function() {
        // 1. Deploy your contract
        let contract_address = deploy_contract();
        let dispatcher = IContractDispatcher { contract_address };

        let contract_to_modify: ContractAddress = 0x456.try_into().unwrap();

        assert(dispatcher.get_balance(contract_to_modify) == 0, 'Wrong balance');

        // 2. Use `interact_with_state` to call contract's internal function
        interact_with_state(
            contract_address,
            || {
                // 3. Get access to the contract's state
                let mut state = Contract::contract_state_for_testing();

                // 4. Call internal function
                state._set_balance(contract_to_modify, 200);
            },
        );

        assert(dispatcher.get_balance(contract_to_modify) == 200, 'Wrong balance');
    }
}

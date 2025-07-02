#[starknet::interface]
pub trait IHelloStarknetExtended<TContractState> {
    fn increase_balance(ref self: TContractState, amount: u256);
    fn get_balance(self: @TContractState) -> u256;
    fn get_caller_info(self: @TContractState, address: starknet::ContractAddress) -> u256;
    fn get_balance_at(self: @TContractState, index: u64) -> u256;
}


#[starknet::contract]
pub mod HelloStarknetExtended {
    use starknet::storage::{
        Map, MutableVecTrait, StoragePathEntry, StoragePointerReadAccess, StoragePointerWriteAccess,
        Vec, VecTrait,
    };
    use starknet::{ContractAddress, get_caller_address};

    #[derive(starknet::Store)]
    struct Owner {
        pub address: ContractAddress,
        pub name: felt252,
    }

    #[storage]
    struct Storage {
        pub owner: Owner,
        pub balance: u256,
        pub balance_records: Vec<u256>,
        pub callers: Map<ContractAddress, u256>,
    }

    #[constructor]
    fn constructor(ref self: ContractState, owner_name: felt252) {
        self
            ._set_owner(
                starknet::get_execution_info().tx_info.account_contract_address, owner_name,
            );
        self.balance_records.push(0);
    }

    #[abi(embed_v0)]
    impl HelloStarknetExtendedImpl of super::IHelloStarknetExtended<ContractState> {
        fn increase_balance(ref self: ContractState, amount: u256) {
            let caller = get_caller_address();
            let value_before = self.callers.entry(caller).read();

            assert(amount != 0, 'Amount cannot be 0');

            self.balance.write(self.balance.read() + amount);
            self.callers.entry(caller).write(value_before + amount);
            self.balance_records.push(self.balance.read());
        }

        fn get_balance(self: @ContractState) -> u256 {
            self.balance.read()
        }

        fn get_caller_info(self: @ContractState, address: ContractAddress) -> u256 {
            self.callers.entry(address).read()
        }

        fn get_balance_at(self: @ContractState, index: u64) -> u256 {
            assert(index < self.balance_records.len(), 'Index out of range');
            self.balance_records.at(index).read()
        }
    }

    #[generate_trait]
    pub impl InternalFunctions of InternalFunctionsTrait {
        fn _set_owner(ref self: ContractState, address: ContractAddress, name: felt252) {
            self.owner.address.write(address);
            self.owner.name.write(name);
        }
    }
}

#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn do_a_panic(self: @TContractState);
    fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
}

#[starknet::contract]
mod HelloStarknet {
    use core::array::ArrayTrait;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[abi(embed_v0)]
    impl IHelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // Increases the balance by the given amount
        fn increase_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(self.balance.read() + amount);
        }

        // Returns the current balance
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }

        // Panics
        fn do_a_panic(self: @ContractState) {
            let mut arr = ArrayTrait::new();
            arr.append('PANIC');
            arr.append('DAYTAH');
            panic(arr);
        }

        // Panics with given array data
        fn do_a_panic_with(self: @ContractState, panic_data: Array<felt252>) {
            panic(panic_data);
        }
    }
}

#[cfg(test)]
mod tests {
    use snforge_std::{declare};
    use snforge_std::cheatcodes::contract_class::DeclareResultTrait;

    #[test]
    fn declare_contract_from_lib() {
        let _ = declare("HelloStarknet").unwrap().contract_class();
        assert(2 == 2, 'Should declare');
    }
}

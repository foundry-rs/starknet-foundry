#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[external(v0)]
    impl IHelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // Increases the balance by the given amount.
        fn increase_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(self.balance.read() + amount);
        }

        // Returns the current balance.
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
    }

    #[external(v0)]
    fn __validate_deploy__(
        self: @ContractState,
        class_hash: felt252,
        contract_address_salt: felt252,
        public_key_: felt252
    ) -> felt252 {
        starknet::VALIDATED
    }
}

mod contract1;

#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn decrease_balance(ref self: TContractState, amount: felt252);
}

#[starknet::contract]
mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[external(v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // Increases the balance by the given amount.
        fn increase_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(self.balance.read() + amount);
        }

        // Decreases the balance by the given amount.
        fn decrease_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(self.balance.read() - amount);
        }
    }
}

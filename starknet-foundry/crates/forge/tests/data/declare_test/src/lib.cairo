mod contract1;

#[starknet::contract]
mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
    }

    // Increases the balance by the given amount.
    #[external]
    fn increase_balance(ref self: ContractState, amount: felt252) {
        self.balance.write(self.balance.read() + amount);
    }

    // Decreases the balance by the given amount.
    #[external]
    fn decrease_balance(ref self: ContractState, amount: felt252) {
        self.balance.write(self.balance.read() - amount);
    }
}

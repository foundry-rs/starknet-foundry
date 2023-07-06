#[starknet::interface]
trait IContract1<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod Contract1 {
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[external(v0)]
    impl Contract1Impl of super::IContract1<ContractState> {
        // Increases the balance by the given amount.
        fn increase_balance(ref self: ContractState, amount: felt252) {
            assert(amount != 0, 'Amount cannot be 0');
            self.balance.write(self.balance.read() + amount);
        }

        // Returns the current balance.
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
    }
}

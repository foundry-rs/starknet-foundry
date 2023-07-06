#[starknet::interface]
trait IContract2<TContractState> {
    fn set_balance(ref self: TContractState, amount: felt252);
}

#[starknet::contract]
mod Contract2 {
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[external(v0)]
    impl Contract2Impl of super::IContract2<ContractState> {
        fn set_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(amount);
        }
    }
}

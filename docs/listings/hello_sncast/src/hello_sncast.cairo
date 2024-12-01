#[starknet::interface]
pub trait IHelloSncast<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn sum_numbers(ref self: TContractState, a: felt252, b: felt252, c: felt252) -> felt252;
}

#[starknet::contract]
mod HelloSncast {
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[abi(embed_v0)]
    impl HelloSncastImpl of super::IHelloSncast<ContractState> {
        fn increase_balance(ref self: ContractState, amount: felt252) {
            assert(amount != 0, 'Amount cannot be 0');
            self.balance.write(self.balance.read() + amount);
        }

        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }

        fn sum_numbers(ref self: ContractState, a: felt252, b: felt252, c: felt252) -> felt252 {
            a + b + c
        }
    }
}

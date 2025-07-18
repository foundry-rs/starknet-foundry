#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn do_a_panic(self: @TContractState);
    fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
}

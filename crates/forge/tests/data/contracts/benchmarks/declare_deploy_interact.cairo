#[starknet::contract]
mod HelloStarknet {
    use box::BoxTrait;
    use starknet::ContractAddressIntoFelt252;
    use starknet::ContractAddress;
    use integer::U64IntoFelt252;
    use option::Option;
    use traits::Into;

    #[storage]
    struct Storage {
        balance: felt252,
    }

    // Increases the balance by the given amount
    #[external(v0)]
    fn increase_balance(ref self: ContractState, amount: felt252) {
        self.balance.write(self.balance.read() + amount);
    }

    // Decreases the balance by the given amount.
    #[external(v0)]
    fn decrease_balance(ref self: ContractState, amount: felt252) {
        self.balance.write(self.balance.read() - amount);
    }

    //Get the balance.
    #[external(v0)]
    fn get_balance(self: @ContractState) -> felt252 {
        self.balance.read()
    }

    #[external(v0)]
    fn interact_with_state(self: @ContractState) -> (felt252, felt252, felt252) {
        let caller_address: felt252 = starknet::get_caller_address().into();
        let block_number = starknet::get_block_info().unbox().block_number;
        let block_timestamp = starknet::get_block_info().unbox().block_timestamp;

        (caller_address, block_number.into(), block_timestamp.into())
    }
}

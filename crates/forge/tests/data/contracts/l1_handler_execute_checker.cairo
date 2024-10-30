#[derive(Copy, Serde, Drop)]
struct L1Data {
    balance: felt252,
    token_id: u256
}

#[starknet::interface]
trait IBalanceToken<TContractState> {
    fn get_balance(self: @TContractState) -> felt252;
    fn get_token_id(self: @TContractState) -> u256;
}

#[starknet::contract]
mod l1_handler_executor {

    use super::{IBalanceToken, L1Data};

    #[storage]
    struct Storage {
        l1_caller: felt252,
        balance: felt252,
        token_id: u256
    }

    #[constructor]
    fn constructor(ref self: ContractState, l1_caller: felt252) {
        self.l1_caller.write(l1_caller);
    }

    #[abi(embed_v0)]
    impl IBalanceTokenImpl of super::IBalanceToken<ContractState> {
        // Returns the current balance
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }

        // Returns the current token id.
        fn get_token_id(self: @ContractState) -> u256 {
            self.token_id.read()
        }
    }

    #[l1_handler]
    fn process_l1_message(ref self: ContractState, from_address: felt252, data: L1Data) {
        assert(from_address == self.l1_caller.read(), 'Unauthorized l1 caller');
        self.balance.write(data.balance);
        self.token_id.write(data.token_id);
    }

    #[l1_handler]
    fn panicking_l1_handler(ref self: ContractState, from_address: felt252) {
        panic(array!['custom', 'panic']);
    }
}

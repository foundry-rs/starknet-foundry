#[starknet::contract]
mod DeployChecker {
    use starknet::ContractAddress;

    #[storage]
    struct Storage {
        balance: felt252,
        caller: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, balance: felt252) -> (ContractAddress, felt252) {
        self.balance.write(balance);
        self.caller.write(starknet::get_caller_address());
        (self.caller.read(), balance)
    }

    #[external(v0)]
    fn get_balance(self: @ContractState) -> felt252 {
        self.balance.read()
    }

    #[external(v0)]
    fn get_caller(self: @ContractState) -> ContractAddress {
        self.caller.read()
    }
}

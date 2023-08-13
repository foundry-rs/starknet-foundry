#[starknet::interface]
trait IMockChecker<TContractState> {
    fn get_thing(ref self: TContractState) -> felt252;
    fn get_thing_wrapper(ref self: TContractState) -> felt252;
    fn get_constant_thing(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod MockChecker {
    use super::IMockChecker;

    #[storage]
    struct Storage {
        stored_thing: felt252
    }

    #[constructor]
    fn constructor(ref self: ContractState, arg1: felt252) {
        self.stored_thing.write(arg1)
    }

    #[external(v0)]
    impl IMockCheckerImpl of super::IMockChecker<ContractState> {
        fn get_thing(ref self: ContractState) -> felt252 {
            self.stored_thing.read()
        }

        fn get_thing_wrapper(ref self: ContractState) -> felt252 {
            self.get_thing()
        }

        fn get_constant_thing(ref self: ContractState) -> felt252 {
            13
        }
    }
}

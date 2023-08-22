use starknet::ContractAddress;

#[starknet::interface]
trait IConstructorMockChecker<TContractState> {
    fn get_stored_thing(ref self: TContractState) -> felt252;
    fn get_constant_thing(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod ConstructorMockChecker {
    use super::IConstructorMockChecker;

    #[storage]
    struct Storage {
        stored_thing: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let const_thing = self.get_constant_thing();
        self.stored_thing.write(const_thing);
    }

    #[external(v0)]
    impl IConstructorMockCheckerImpl of super::IConstructorMockChecker<ContractState> {
        fn get_constant_thing(ref self: ContractState) -> felt252 {
            13
        }

        fn get_stored_thing(ref self: ContractState) -> felt252 {
            self.stored_thing.read()
        }
    }
}

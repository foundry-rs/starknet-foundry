#[starknet::interface]
trait IConstructorSimple<TContractState> {
    fn add_to_number(ref self: TContractState, number: felt252) -> felt252;
    fn get_number(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod ConstructorSimple {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        number: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState, number: felt252) {
        self.number.write(number);
    }

    #[abi(embed_v0)]
    impl ConstructorSimpleImpl of super::IConstructorSimple<ContractState> {
        fn add_to_number(ref self: ContractState, number: felt252) -> felt252 {
            let new_number = self.number.read() + number;
            self.number.write(new_number);
            new_number
        }

        fn get_number(self: @ContractState) -> felt252 {
            self.number.read()
        }
    }
}

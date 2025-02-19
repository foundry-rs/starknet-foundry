#[starknet::interface]
trait IConstructorSimple2<TContractState> {
    fn add_to_number(ref self: TContractState, number: felt252) -> felt252;
    fn get_number(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod ConstructorSimple2 {
    use core::starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        number: felt252
    }

    #[constructor]
    fn constructor(ref self: ContractState, number: felt252, number2: felt252) {
        self.number.write(number + number2);
    }

    #[abi(embed_v0)]
    impl ConstructorSimple2Impl of super::IConstructorSimple2<ContractState> {
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

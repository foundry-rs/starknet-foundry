#[derive(Serde, Drop)]
pub struct SimpleStruct {
    a: felt252
}

#[derive(Serde, Drop)]
pub struct NestedStructWithField {
    a: SimpleStruct,
    b: felt252
}

#[derive(Serde, Drop)]
pub enum Enum {
    One: (),
    Two: u128,
    Three: NestedStructWithField
}

#[starknet::interface]
pub trait IDataTransformerContract<TContractState> {
    fn tuple_fn(self: @TContractState, a: (felt252, u8, Enum));
    fn nested_struct_fn(self: @TContractState, a: NestedStructWithField);
    fn complex_fn(
        self: @TContractState,
        arr: Array<Array<felt252>>,
        one: u8,
        two: i8,
        three: ByteArray,
        four: (felt252, u32),
        five: bool,
        six: u256
    );
}


#[starknet::contract]
pub mod DataTransformerContract {
    use super::{NestedStructWithField, Enum};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl DataTransformerContractImpl of super::IDataTransformerContract<ContractState> {
        fn tuple_fn(self: @ContractState, a: (felt252, u8, Enum)) {}

        fn nested_struct_fn(self: @ContractState, a: NestedStructWithField) {}

        fn complex_fn(
            self: @ContractState,
            arr: Array<Array<felt252>>,
            one: u8,
            two: i8,
            three: ByteArray,
            four: (felt252, u32),
            five: bool,
            six: u256
        ) {}
    }
}

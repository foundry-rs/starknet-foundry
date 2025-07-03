#[derive(Serde, Drop)]
pub struct SimpleStruct {
    a: felt252,
}

#[derive(Serde, Drop)]
pub struct NestedStructWithField {
    a: SimpleStruct,
    b: felt252,
}

#[derive(Serde, Drop)]
pub enum Enum {
    One: (),
    Two: u128,
    Three: NestedStructWithField,
}

#[derive(Serde, Drop)]
pub struct ComplexStruct {
    a: NestedStructWithField,
    b: felt252,
    c: u8,
    d: i32,
    e: Enum,
    f: ByteArray,
    g: Array<felt252>,
    h: u256,
    i: (i128, u128),
}

#[derive(Serde, Drop)]
pub struct BitArray {
    bit: felt252,
}

#[starknet::interface]
pub trait IDataTransformer<TContractState> {
    fn simple_fn(ref self: TContractState, a: felt252) -> felt252;
    fn u256_fn(ref self: TContractState, a: u256) -> u256;
    fn signed_fn(ref self: TContractState, a: i32) -> i32;
    fn unsigned_fn(ref self: TContractState, a: u32) -> u32;
    fn tuple_fn(ref self: TContractState, a: (felt252, u8, Enum)) -> (felt252, u8, Enum);
    fn complex_fn(
        ref self: TContractState,
        arr: Array<Array<felt252>>,
        one: u8,
        two: i16,
        three: ByteArray,
        four: (felt252, u32),
        five: bool,
        six: u256,
    );
    fn simple_struct_fn(ref self: TContractState, a: SimpleStruct) -> SimpleStruct;
    fn nested_struct_fn(
        ref self: TContractState, a: NestedStructWithField,
    ) -> NestedStructWithField;
    fn enum_fn(ref self: TContractState, a: Enum) -> Enum;
    fn complex_struct_fn(ref self: TContractState, a: ComplexStruct) -> ComplexStruct;
    fn external_struct_fn(
        ref self: TContractState, a: BitArray, b: alexandria_data_structures::bit_array::BitArray,
    ) -> (BitArray, alexandria_data_structures::bit_array::BitArray);
    fn span_fn(ref self: TContractState, a: Span<felt252>) -> Span<felt252>;
    fn multiple_signed_fn(ref self: TContractState, a: i32, b: i8);
    fn no_args_fn(ref self: TContractState);
}

#[starknet::contract]
mod DataTransformer {
    use core::starknet::ContractAddress;
    use super::*;

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState, init_owner: ContractAddress) {}

    #[abi(embed_v0)]
    impl DataTransformerImpl of super::IDataTransformer<ContractState> {
        fn simple_fn(ref self: ContractState, a: felt252) -> felt252 {
            a
        }
        fn u256_fn(ref self: ContractState, a: u256) -> u256 {
            a
        }
        fn signed_fn(ref self: ContractState, a: i32) -> i32 {
            a
        }
        fn unsigned_fn(ref self: ContractState, a: u32) -> u32 {
            a
        }
        fn tuple_fn(ref self: ContractState, a: (felt252, u8, Enum)) -> (felt252, u8, Enum) {
            a
        }
        fn complex_fn(
            ref self: ContractState,
            arr: Array<Array<felt252>>,
            one: u8,
            two: i16,
            three: ByteArray,
            four: (felt252, u32),
            five: bool,
            six: u256,
        ) {}
        fn simple_struct_fn(ref self: ContractState, a: SimpleStruct) -> SimpleStruct {
            a
        }
        fn nested_struct_fn(
            ref self: ContractState, a: NestedStructWithField,
        ) -> NestedStructWithField {
            a
        }
        fn enum_fn(ref self: ContractState, a: Enum) -> Enum {
            a
        }
        fn complex_struct_fn(ref self: ContractState, a: ComplexStruct) -> ComplexStruct {
            a
        }
        fn external_struct_fn(
            ref self: ContractState,
            a: BitArray,
            b: alexandria_data_structures::bit_array::BitArray,
        ) -> (BitArray, alexandria_data_structures::bit_array::BitArray) {
            (a, b)
        }
        fn span_fn(ref self: ContractState, a: Span<felt252>) -> Span<felt252> {
            a
        }
        fn multiple_signed_fn(ref self: ContractState, a: i32, b: i8) {}
        fn no_args_fn(ref self: ContractState) {}
    }
}

#[starknet::contract]
mod DataTransformerNoConstructor {
    #[storage]
    struct Storage {}
}

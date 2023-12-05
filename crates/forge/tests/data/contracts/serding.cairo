#[derive(Drop, Serde)]
struct NestedStruct {
    d: felt252,
}

#[derive(Drop, Serde)]
struct CustomStruct {
    a: felt252,
    b: felt252,
    c: NestedStruct,
}

#[derive(Drop, Serde)]
struct AnotherCustomStruct {
    e: felt252,
}

#[starknet::interface]
trait ISerding<TContractState> {
    fn add_multiple_parts(
        self: @TContractState,
        custom_struct: CustomStruct,
        another_struct: AnotherCustomStruct,
        standalone_arg: felt252
    ) -> felt252;
}

#[starknet::contract]
mod Serding {
    use super::{CustomStruct, AnotherCustomStruct};
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl SerdingImpl of super::ISerding<ContractState> {
        fn add_multiple_parts(
            self: @ContractState,
            custom_struct: CustomStruct,
            another_struct: AnotherCustomStruct,
            standalone_arg: felt252
        ) -> felt252 {
            custom_struct.a + custom_struct.b + custom_struct.c.d + another_struct.e + standalone_arg
        }
    }

}

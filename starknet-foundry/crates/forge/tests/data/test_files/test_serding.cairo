use array::ArrayTrait;
use result::ResultTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;

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
trait ISerding<T> {
    fn add_multiple_parts(
        self: @T,
        custom_struct: CustomStruct,
        another_struct: AnotherCustomStruct,
        standalone_arg: felt252
    ) -> felt252;
}

#[test]
fn test_serding() {
    let class_hash = declare('Serding').unwrap();
    let prepared = PreparedContract {
        class_hash, constructor_calldata: @ArrayTrait::new()
    };
    let contract_address = deploy(prepared).unwrap().try_into().unwrap();

    let safe_dispatcher = ISerdingSafeDispatcher {
        contract_address
    };

    let ns = NestedStruct { d: 1 };
    let cs = CustomStruct { a: 2, b: 3, c: ns };
    let acs = AnotherCustomStruct { e: 4 };
    let standalone_arg = 5;

    let result = safe_dispatcher.add_multiple_parts(cs, acs, standalone_arg).unwrap();

    assert(result == 1 + 2 + 3 + 4 + 5, 'Invalid sum');
}

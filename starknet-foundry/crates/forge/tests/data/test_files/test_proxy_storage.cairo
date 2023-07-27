use array::ArrayTrait;
use result::ResultTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;

#[derive(Drop, Serde, PartialEq, Copy)]
struct NestedStruct {
    d: felt252,
}

#[derive(Drop, Serde, PartialEq, Copy)]
struct CustomStruct {
    a: felt252,
    b: felt252,
    c: NestedStruct,
}

fn deploy_contract(name: felt252) -> ContractAddress {
    let class_hash = declare(name);
    let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
    let contract_address = deploy(prepared).unwrap();

    contract_address.try_into().unwrap()
}


#[starknet::interface]
trait ICaller<T> {
    fn call_executor(
        self: @T, executor_address: starknet::ContractAddress, custom_struct: CustomStruct
    ) -> felt252;
}

#[starknet::interface]
trait IExecutor<T> {
    fn read_storage(ref self: T) -> CustomStruct;
}

#[test]
fn test_proxy_storage() {
    let caller_address = deploy_contract('Caller');
    let executor_address = deploy_contract('Executor');

    let caller_dispatcher = ICallerSafeDispatcher { contract_address: caller_address };
    let executor_dispatcher = IExecutorSafeDispatcher { contract_address: executor_address };

    let ns = NestedStruct { d: 6 };
    let cs = CustomStruct { a: 2, b: 3, c: ns };

    let result = caller_dispatcher.call_executor(executor_address, cs).unwrap();

    assert(result == 6 + 5, 'Invalid result');

    let storage_after = executor_dispatcher.read_storage().unwrap();

    assert(storage_after == cs, 'Invalid storage');
}

use core::starknet::testing::cheatcode;
use core::starknet::ContractAddress;

#[derive(Drop, Serde, PartialEq)]
struct CallEntryPoint {
    entry_point_type: EntryPointType,
    entry_point_selector: felt252,
    calldata: Array<felt252>,
    contract_address: ContractAddress,
    caller_address: ContractAddress,
    call_type: CallType,
}

#[derive(Drop, Serde, PartialEq)]
enum EntryPointType {
    Constructor,
    External,
    L1Handler,
}

#[derive(Drop, Serde, PartialEq)]
enum CallType {
    Call,
    Delegate,
}

fn get_last_call_trace() -> Array<CallEntryPoint> {
    let mut output = cheatcode::<'get_last_call_trace'>(array![].span());
    Serde::<Array<CallEntryPoint>>::deserialize(ref output).unwrap()
}

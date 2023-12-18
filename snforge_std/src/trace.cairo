use starknet::testing::cheatcode;
use starknet::ContractAddress;
use super::PrintTrait;

#[derive(Drop, Serde, PartialEq)]
struct CallEntryPoint {
    entry_point_type: EntryPointType,
    entry_point_selector: felt252,
    calldata: Array<felt252>,
    storage_address: ContractAddress,
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

impl CallEntryPointPrintImpl of PrintTrait<CallEntryPoint> {
    fn print(self: @CallEntryPoint) {
        'Entry Point Type'.print();
        match self.entry_point_type {
            EntryPointType::Constructor => 'Constructor'.print(),
            EntryPointType::External => 'External'.print(),
            EntryPointType::L1Handler => 'L1 Handler'.print(),
        };

        'Entry Point Selector'.print();
        self.entry_point_selector.print();

        'Calldata'.print();
        self.calldata.print();

        'Storage Address'.print();
        self.storage_address.print();

        'Caller Address'.print();
        self.caller_address.print();

        'Call Type'.print();
        match self.call_type {
            CallType::Call => 'Call'.print(),
            CallType::Delegate => 'Delegate'.print(),
        };
    }
}

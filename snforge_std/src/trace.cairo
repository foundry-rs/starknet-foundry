use core::starknet::testing::cheatcode;
use core::starknet::ContractAddress;
use core::fmt::{Display, Debug, Formatter, Error};

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

impl DisplayEntryPointType of Display<EntryPointType> {
    fn fmt(self: @EntryPointType, ref f: Formatter) -> Result<(), Error> {
        let str: ByteArray = match self {
            EntryPointType::Constructor => "Constructor",
            EntryPointType::External => "External",
            EntryPointType::L1Handler => "L1 Handler",
        };
        f.buffer.append(@str);
        Result::Ok(())
    }
}

impl DisplayCallType of Display<CallType> {
    fn fmt(self: @CallType, ref f: Formatter) -> Result<(), Error> {
        let str: ByteArray = match self {
            CallType::Call => "Call",
            CallType::Delegate => "Delegate",
        };
        f.buffer.append(@str);
        Result::Ok(())
    }
}

impl DisplayCallEntryPoint of Display<CallEntryPoint> {
    fn fmt(self: @CallEntryPoint, ref f: Formatter) -> Result<(), Error> {
        write!(f, "Entry point type: ")?;
        Display::fmt(self.entry_point_type, ref f)?;
        write!(f, "\nSelector: ")?;
        Display::fmt(self.entry_point_selector, ref f)?;
        write!(f, "\nCalldata: ")?;
        Debug::fmt(self.calldata, ref f)?;
        write!(f, "\nStorage address: ")?;
        Debug::fmt(self.contract_address, ref f)?;
        write!(f, "\nCaller address: ")?;
        Debug::fmt(self.caller_address, ref f)?;
        write!(f, "\nCall type: ")?;
        Display::fmt(self.call_type, ref f)
    }
}

impl DisplayArrayCallEntryPoint of Display<Array<CallEntryPoint>> {
    fn fmt(self: @Array<CallEntryPoint>, ref f: Formatter) -> Result<(), Error> {
        let mut i = 0;
        loop {
            if i == self.len() {
                break;
            }
            write!(f, "\n").unwrap();
            Display::fmt(self[i], ref f).unwrap();
            write!(f, "\n").unwrap();
            i += 1;
        };
        Result::Ok(())
    }
}

use core::starknet::testing::cheatcode;
use core::starknet::ContractAddress;

#[derive(Drop, Serde, PartialEq)]
struct CallTrace {
    entry_point: CallEntryPoint,
    nested_calls: Array<CallTrace>,
}

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

fn get_call_trace() -> CallTrace {
    let mut output = cheatcode::<'get_call_trace'>(array![].span());
    Serde::deserialize(ref output).unwrap()
}

use core::fmt::{Display, Formatter, Error, Debug};

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

impl DisplayCallTrace of Display<CallTrace> {
    fn fmt(self: @CallTrace, ref f: Formatter) -> Result<(), Error> {
        Display::fmt(@IndentedCallTrace { struct_ref: self, base_indents: 0 }, ref f).unwrap();
        Result::Ok(())
    }
}

#[derive(Drop)]
struct IndentedCallTrace {
    struct_ref: @CallTrace,
    base_indents: u8,
}

#[derive(Drop)]
struct IndentedCallTraceArray {
    struct_ref: @Array<CallTrace>,
    base_indents: u8,
}

#[derive(Drop)]
struct IndentedEntryPoint {
    struct_ref: @CallEntryPoint,
    base_indents: u8,
}


impl DisplayIndentedCallTrace of Display<IndentedCallTrace> {
    fn fmt(self: @IndentedCallTrace, ref f: Formatter) -> Result<(), Error> {
        Display::fmt(
            @IndentedEntryPoint {
                base_indents: *self.base_indents, struct_ref: *self.struct_ref.entry_point
            },
            ref f
        )
            .unwrap();
        write!(f, "\n").unwrap();
        write_indents_to_formatter(*self.base_indents, ref f);
        write!(f, "Nested Calls: [").unwrap();
        if (*self.struct_ref.nested_calls).len() > 0 {
            write!(f, "\n").unwrap();
            Display::fmt(
                @IndentedCallTraceArray {
                    base_indents: (*self.base_indents) + 1,
                    struct_ref: *self.struct_ref.nested_calls
                },
                ref f
            )
                .unwrap();
            write!(f, "\n").unwrap();
            write_indents_to_formatter(*self.base_indents, ref f);
        }

        write!(f, "]").unwrap();
        Result::Ok(())
    }
}

impl DisplayIndentedCallTraceArray of Display<IndentedCallTraceArray> {
    fn fmt(self: @IndentedCallTraceArray, ref f: Formatter) -> Result<(), Error> {
        let mut i: u32 = 0;
        let trace_len = (*self.struct_ref).len();
        while i < trace_len {
            write_indents_to_formatter(*self.base_indents, ref f);
            write!(f, "(\n").unwrap();

            Display::fmt(
                @IndentedCallTrace {
                    base_indents: *self.base_indents + 1, struct_ref: (*self.struct_ref)[i]
                },
                ref f
            )
                .unwrap();
            write!(f, "\n").unwrap();
            write_indents_to_formatter(*self.base_indents, ref f);
            write!(f, ")").unwrap();

            i = i + 1;
            if i != trace_len {
                write!(f, ",\n").unwrap();
            }
        };

        Result::Ok(())
    }
}

impl DisplayIndentedEntryPoint of Display<IndentedEntryPoint> {
    fn fmt(self: @IndentedEntryPoint, ref f: Formatter) -> Result<(), Error> {
        write_indents_to_formatter(*self.base_indents, ref f);
        write!(f, "Entry point type: ")?;
        Display::fmt(*self.struct_ref.entry_point_type, ref f)?;

        write!(f, "\n")?;
        write_indents_to_formatter(*self.base_indents, ref f);
        write!(f, "Selector: ")?;
        Display::fmt(*self.struct_ref.entry_point_selector, ref f)?;

        write!(f, "\n")?;
        write_indents_to_formatter(*self.base_indents, ref f);
        write!(f, "Calldata: ")?;
        Debug::fmt(*self.struct_ref.calldata, ref f)?;

        write!(f, "\n")?;
        write_indents_to_formatter(*self.base_indents, ref f);
        write!(f, "Storage address: ")?;
        Debug::fmt(*self.struct_ref.contract_address, ref f)?;

        write!(f, "\n")?;
        write_indents_to_formatter(*self.base_indents, ref f);
        write!(f, "Caller address: ")?;
        Debug::fmt(*self.struct_ref.caller_address, ref f)?;

        write!(f, "\n")?;
        write_indents_to_formatter(*self.base_indents, ref f);
        write!(f, "Call type: ")?;
        Display::fmt(*self.struct_ref.call_type, ref f)?;

        Result::Ok(())
    }
}


fn write_indents_to_formatter(indents: u8, ref f: Formatter) {
    let mut i: u8 = 0;
    while i < indents {
        write!(f, "    ").unwrap();
        i = i + 1;
    }
}

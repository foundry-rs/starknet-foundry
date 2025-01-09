use core::starknet::ContractAddress;
use super::_cheatcode::typed_checked_cheatcode;

/// Tree-like structure which contains all of the starknet calls and sub-calls along with the
/// results
#[derive(Drop, Serde, PartialEq, Clone, Debug)]
pub struct CallTrace {
    pub entry_point: CallEntryPoint,
    /// All the calls that happened in the scope of `entry_point`
    pub nested_calls: Array<CallTrace>,
    pub result: CallResult,
}

/// A single function entry point summary
#[derive(Drop, Serde, PartialEq, Clone, Debug)]
pub struct CallEntryPoint {
    pub entry_point_type: EntryPointType,
    /// Hashed selector of the invoked function
    pub entry_point_selector: felt252,
    /// Serialized arguments calldata
    pub calldata: Array<felt252>,
    /// Contract address targeted by the call
    pub contract_address: ContractAddress,
    /// Address that the call originates from
    pub caller_address: ContractAddress,
    pub call_type: CallType,
}

/// Type of the function being invoked
#[derive(Drop, Serde, PartialEq, Clone, Debug)]
pub enum EntryPointType {
    /// Constructor of a contract
    Constructor,
    /// Contract interface entry point
    External,
    /// An entrypoint for handling messages from L1
    L1Handler,
}

/// Denotes type of the call
#[derive(Drop, Serde, PartialEq, Clone, Debug)]
pub enum CallType {
    /// Regular call
    Call,
    /// Library call
    Delegate,
}

/// Result of a contract or a library call
#[derive(Drop, Serde, PartialEq, Clone, Debug)]
pub enum CallResult {
    /// A successful call with it's result
    Success: Array<felt252>,
    /// A failed call along with it's panic data
    Failure: CallFailure,
}

/// Represents a pre-processed failure of a call
#[derive(Drop, Serde, PartialEq, Clone, Debug)]
pub enum CallFailure {
    /// Contains raw panic data
    Panic: Array<felt252>,
    /// Contains panic data in parsed form, if parsing is applicable
    Error: ByteArray,
}

/// Returns current call trace of the test, up to the last call made to a contract
pub fn get_call_trace() -> CallTrace {
    typed_checked_cheatcode::<'get_call_trace', CallTrace>(array![].span())
}

use core::fmt::{Display, Formatter, Error, Debug};

impl DisplayCallResult of Display<CallResult> {
    fn fmt(self: @CallResult, ref f: Formatter) -> Result<(), Error> {
        match self {
            CallResult::Success(val) => {
                write!(f, "Success: ")?;
                Debug::fmt(val, ref f)?;
            },
            CallResult::Failure(call_failure) => {
                write!(f, "Failure: ")?;

                match call_failure {
                    CallFailure::Panic(val) => { Debug::fmt(val, ref f)?; },
                    CallFailure::Error(msg) => { Display::fmt(msg, ref f)?; },
                };
            },
        };
        Result::Ok(())
    }
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

impl DisplayCallTrace of Display<CallTrace> {
    fn fmt(self: @CallTrace, ref f: Formatter) -> Result<(), Error> {
        Display::fmt(@IndentedCallTrace { struct_ref: self, base_indents: 0 }, ref f).unwrap();
        Result::Ok(())
    }
}

#[derive(Drop)]
struct Indented<T> {
    struct_ref: @T,
    base_indents: u8,
}

type IndentedEntryPoint = Indented<CallEntryPoint>;
type IndentedCallTraceArray = Indented<Array<CallTrace>>;
type IndentedCallTrace = Indented<CallTrace>;
type IndentedCallResult = Indented<CallResult>;


impl DisplayIndentedCallTrace of Display<Indented<CallTrace>> {
    fn fmt(self: @Indented<CallTrace>, ref f: Formatter) -> Result<(), Error> {
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

        write!(f, "\n").unwrap();
        Display::fmt(
            @IndentedCallResult {
                base_indents: *self.base_indents, struct_ref: *self.struct_ref.result
            },
            ref f
        )
            .unwrap();

        Result::Ok(())
    }
}

impl DisplayIndentedCallTraceArray of Display<Indented<Array<CallTrace>>> {
    fn fmt(self: @Indented<Array<CallTrace>>, ref f: Formatter) -> Result<(), Error> {
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

impl DisplayIndentedEntryPoint of Display<Indented<CallEntryPoint>> {
    fn fmt(self: @Indented<CallEntryPoint>, ref f: Formatter) -> Result<(), Error> {
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

impl DisplayIndentedCallResult of Display<Indented<CallResult>> {
    fn fmt(self: @Indented<CallResult>, ref f: Formatter) -> Result<(), Error> {
        write_indents_to_formatter(*self.base_indents, ref f);
        write!(f, "Call Result: ")?;
        Display::fmt(*self.struct_ref, ref f)?;

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

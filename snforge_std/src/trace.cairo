use core::starknet::testing::cheatcode;
use core::starknet::ContractAddress;

#[derive(Drop)]
struct CallTrace {
    entry_point: CallEntryPoint,
    nested_calls: Array<CallTrace>,
}

impl SerdeCallTrace of Serde<CallTrace> {
    fn serialize(self: @CallTrace, ref output: Array<felt252>) {
        self.entry_point.serialize(ref output);
        self.nested_calls.len().serialize(ref output);

        let mut span = self.nested_calls.span();
        loop {
            match span.pop_front() {
                Option::Some(nested_call) => { nested_call.serialize(ref output); },
                Option::None => { break; },
            }
        };
    }

    fn deserialize(ref serialized: Span<felt252>) -> Option<CallTrace> {
        let entry_point = Serde::deserialize(ref serialized)?;
        let mut res = CallTrace { entry_point, nested_calls: array![] };

        let mut len = *serialized.pop_front()?;

        let mut failed = false;
        loop {
            if len == 0 {
                break;
            }

            let call_trace = Serde::deserialize(ref serialized);
            let call_trace = match call_trace {
                Option::Some(call_trace) => call_trace,
                Option::None => {
                    failed = true;
                    break;
                },
            };
            res.nested_calls.append(call_trace);

            len -= 1;
        };

        if failed {
            Option::None
        } else {
            Option::Some(res)
        }
    }
}

impl PartialEqCallTrace of PartialEq<CallTrace> {
    fn eq(lhs: @CallTrace, rhs: @CallTrace) -> bool {
        if lhs.entry_point != rhs.entry_point {
            return false;
        }

        let mut lhs_span = lhs.nested_calls.span();
        let mut rhs_span = rhs.nested_calls.span();

        loop {
            match lhs_span.pop_front() {
                Option::Some(lhs_call_trace) => {
                    match rhs_span.pop_front() {
                        Option::Some(rhs_call_trace) => {
                            if rhs_call_trace != lhs_call_trace {
                                break false;
                            }
                        },
                        Option::None => { break false; }
                    }
                },
                Option::None => { break rhs_span.pop_front().is_none(); },
            }
        }
    }

    fn ne(lhs: @CallTrace, rhs: @CallTrace) -> bool {
        !(lhs == rhs)
    }
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

# `get_call_trace`

```rust
fn get_call_trace() -> CallTrace;
```

(For whole structure definition, please refer to [`snforge-std` source](https://github.com/foundry-rs/starknet-foundry/tree/v0.16.0/snforge_std))

Gets current call trace of the test, up to the last call made to a contract.

## Example call trace
```cairo
use snforge_std::trace::{CallTrace, CallEntryPoint, CallType, EntryPointType, get_call_trace};
...

let ctrace = CallTrace {
    entry_point: CallEntryPoint {
        entry_point_type: EntryPointType::External,
        entry_point_selector: test_selector(),
        calldata: array![],
        contract_address: test_address(),
        caller_address: 0.try_into().unwrap(),
        call_type: CallType::Call,
    },
    nested_calls: array![
        CallTrace {
            entry_point: CallEntryPoint {
                entry_point_type: EntryPointType::Constructor,
                entry_point_selector: selector!("constructor"),
                calldata: array![346.into()],
                contract_address: 123.try_into().unwrap(),
                caller_address: test_address(),
                call_type: CallType::Call,
            },
            nested_calls: array![
                CallTrace {
                    entry_point: CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("other_entrypoint"),
                        calldata: array![1],
                        contract_address: 567.try_into().unwrap(),
                        caller_address: 123.try_into().unwrap(),
                        call_type: CallType::Call,
                    },
                    nested_calls: array![]
                }
            ],
        },
        CallTrace {
            entry_point: CallEntryPoint {
                entry_point_type: EntryPointType::L1Handler,
                entry_point_selector: selector!("handle_l1_message"),
                calldata: array![123, 567.into()],
                contract_address: 346.try_into().unwrap(),
                caller_address: 0.try_into().unwrap(),
                call_type: CallType::Call,
            },
            nested_calls: array![]
        }
    ]
};
```

The topmost-call is representing the test call, which will always be present if you're running a test.
It can have nested `CallTrace` - it's an array of subsequent traces made in scope of the call.

The whole structure is represented as a tree of calls, in which each contract interaction (`constructor` call, `l1_handler` call, library or a regular contract `call` via dispatcher) is a new execution scope - thus resulting in a new nested trace.

## Displaying the trace

The `CallTrace` structure implements a `Display` trait, for a pretty-print with indentations:

```cairo
println!("{}", get_call_trace());
//   ...
//   Entry point type: External
//   Selector: [..]
//   Calldata: []
//   Storage address: [..]
//   Caller address: 0
//   Call type: Call
//   Nested Calls: [
//       (
//           Entry point type: External
//           Selector: [..]
//           Calldata: [..]
//           Storage address: [..]
//           Caller address: [..]
//           Call type: Call
//           Nested Calls: [
//               (
//                   Entry point type: External
//                   Selector: [..]
//                   Calldata: [0]
//                   Storage address: [..]
//                   Caller address: [..]
//                   Call type: Call
//                   Nested Calls: []
//               )
//           ]
//       )
//   ]
```
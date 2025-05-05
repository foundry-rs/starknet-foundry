use crate::Trace;
use crate::trace::types::{
    CallerAddress, ContractName, ContractTrace, Selector, StorageAddress, TestName, TraceInfo,
};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::{CallTrace, CallTraceNode};
use std::cell::RefCell;
use std::rc::Rc;

pub fn collect_trace(
    call_trace: &CallTrace,
    contracts_data: &ContractsData,
    test_name: String,
) -> Trace {
    Trace {
        test_name: TestName(test_name),
        nested_calls: collect_nested_calls(call_trace, contracts_data),
    }
}

fn collect_contract_trace(
    call_trace: &Rc<RefCell<CallTrace>>,
    contracts_data: &ContractsData,
) -> ContractTrace {
    let call_trace = call_trace.borrow();
    let nested_calls = collect_nested_calls(&call_trace, contracts_data);

    let trace_info = TraceInfo {
        contract_name: collect_contract_name(&call_trace, contracts_data),
        entry_point_type: call_trace.entry_point.entry_point_type,
        calldata: call_trace.entry_point.calldata.clone(),
        storage_address: StorageAddress(call_trace.entry_point.storage_address),
        caller_address: CallerAddress(call_trace.entry_point.caller_address),
        call_type: call_trace.entry_point.call_type,
        nested_calls,
        call_result: call_trace.result.clone(),
    };

    ContractTrace {
        selector: collect_selector(&call_trace, contracts_data),
        trace_info,
    }
}

fn collect_nested_calls(
    call_trace: &CallTrace,
    contracts_data: &ContractsData,
) -> Vec<ContractTrace> {
    call_trace
        .nested_calls
        .iter()
        .filter_map(CallTraceNode::extract_entry_point_call)
        .map(|call_trace| collect_contract_trace(call_trace, contracts_data))
        .collect()
}

fn collect_contract_name(call_trace: &CallTrace, contracts_data: &ContractsData) -> ContractName {
    contracts_data
        .get_contract_name(
            &call_trace
                .entry_point
                .class_hash
                .expect("class_hash should be set in `fn execute_call_entry_point` in cheatnet"),
        )
        .cloned()
        .map(ContractName)
        .expect("contract name should be present in `ContractsData`")
}

fn collect_selector(call_trace: &CallTrace, contracts_data: &ContractsData) -> Selector {
    contracts_data
        .get_function_name(&call_trace.entry_point.entry_point_selector)
        .cloned()
        .map(Selector)
        .expect("selector should be present in `ContractsData`")
}

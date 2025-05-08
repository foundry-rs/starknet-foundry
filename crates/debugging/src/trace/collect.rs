use crate::Trace;
use crate::trace::types::{
    CallerAddress, ContractName, ContractTrace, Selector, StorageAddress, TestName, TraceInfo,
    TransformedCallResult, TransformedCalldata,
};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult as CheatnetCallResult,
};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::{CallTrace, CallTraceNode};
use data_transformer::{reverse_transform_input, reverse_transform_output};
use serde_json::Value;
use starknet::core::types::contract::AbiEntry;
use starknet_api::execution_utils::format_panic_data;
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
    let contract_name = collect_contract_name(&call_trace, contracts_data);
    let abi = collect_abi(&contract_name, contracts_data);

    let trace_info = TraceInfo {
        contract_name,
        entry_point_type: call_trace.entry_point.entry_point_type,
        calldata: collect_transformed_calldata(&call_trace, &abi),
        storage_address: StorageAddress(call_trace.entry_point.storage_address),
        caller_address: CallerAddress(call_trace.entry_point.caller_address),
        call_type: call_trace.entry_point.call_type,
        nested_calls,
        call_result: collect_transformed_call_result(&call_trace, &abi),
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

fn collect_abi(contract_name: &ContractName, contracts_data: &ContractsData) -> Vec<AbiEntry> {
    let artifacts = contracts_data
        .get_artifacts(&contract_name.0)
        .expect("artifact should be present in `ContractsData`");

    let abi = serde_json::from_str::<Value>(&artifacts.sierra)
        .expect("sierra should be valid json")
        .get_mut("abi")
        .expect("abi value should be present in sierra")
        .take();

    serde_json::from_value(abi).expect("abi value should be valid ABI")
}

fn collect_transformed_calldata(call_trace: &CallTrace, abi: &[AbiEntry]) -> TransformedCalldata {
    TransformedCalldata(
        reverse_transform_input(
            &call_trace.entry_point.calldata.0,
            abi,
            &call_trace.entry_point.entry_point_selector.0,
        )
        .expect("calldata should be successfully transformed"),
    )
}

fn collect_transformed_call_result(
    call_trace: &CallTrace,
    abi: &[AbiEntry],
) -> TransformedCallResult {
    TransformedCallResult(match &call_trace.result {
        CheatnetCallResult::Success { ret_data } => {
            let ret_data = reverse_transform_output(
                ret_data,
                abi,
                &call_trace.entry_point.entry_point_selector.0,
            )
            .expect("call result should be successfully transformed");
            format_result_message("success", &ret_data)
        }
        CheatnetCallResult::Failure(failure) => match failure {
            CallFailure::Panic { panic_data } => {
                format_result_message("panic", &format_panic_data(panic_data))
            }
            CallFailure::Error { msg } => format_result_message("error", &msg.to_string()),
        },
    })
}

fn format_result_message(tag: &str, message: &str) -> String {
    if message.is_empty() {
        tag.to_string()
    } else {
        format!("{tag}: {message}")
    }
}

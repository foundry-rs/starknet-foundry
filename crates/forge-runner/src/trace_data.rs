use std::fs;
use std::path::PathBuf;

// Will be provided by profiler crate in the future
// This module will be removed!
use cheatnet::state::CallTrace as InternalCallTrace;
use serde::{Deserialize, Serialize};
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::transaction::Calldata;

use crate::test_case_summary::{Single, TestCaseSummary};

pub const TRACE_DIR: &str = "snfoundry_trace";

/// Tree structure representing trace of a call.
#[derive(Debug, Clone, Serialize)]
pub struct CallTrace {
    pub entry_point: CallEntryPoint,
    pub nested_calls: Vec<CallTrace>,
}

impl From<InternalCallTrace> for CallTrace {
    fn from(value: InternalCallTrace) -> Self {
        CallTrace {
            entry_point: CallEntryPoint::from(value.entry_point),
            nested_calls: value
                .nested_calls
                .into_iter()
                .map(|c| CallTrace::from(c.borrow().clone()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct CallEntryPoint {
    pub class_hash: Option<ClassHash>,
    pub code_address: Option<ContractAddress>,
    pub entry_point_type: EntryPointType,
    pub entry_point_selector: EntryPointSelector,
    pub calldata: Calldata,
    pub storage_address: ContractAddress,
    pub caller_address: ContractAddress,
    pub call_type: CallType,
    pub initial_gas: u64,
}

impl From<blockifier::execution::entry_point::CallEntryPoint> for CallEntryPoint {
    fn from(value: blockifier::execution::entry_point::CallEntryPoint) -> Self {
        let blockifier::execution::entry_point::CallEntryPoint {
            class_hash,
            code_address,
            entry_point_type,
            entry_point_selector,
            calldata,
            storage_address,
            caller_address,
            call_type,
            initial_gas,
        } = value;

        CallEntryPoint {
            class_hash,
            code_address,
            entry_point_type,
            entry_point_selector,
            calldata,
            storage_address,
            caller_address,
            call_type: call_type.into(),
            initial_gas,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum CallType {
    #[default]
    Call = 0,
    Delegate = 1,
}

impl From<blockifier::execution::entry_point::CallType> for CallType {
    fn from(value: blockifier::execution::entry_point::CallType) -> Self {
        match value {
            blockifier::execution::entry_point::CallType::Call => CallType::Call,
            blockifier::execution::entry_point::CallType::Delegate => CallType::Delegate,
        }
    }
}

pub fn save_trace_data(summary: &TestCaseSummary<Single>) {
    if let TestCaseSummary::Passed {
        name, trace_data, ..
    } = summary
    {
        let serialized_trace =
            serde_json::to_string(trace_data).expect("Failed to serialize call trace");
        let dir_to_save_trace = PathBuf::from(TRACE_DIR);
        fs::create_dir_all(&dir_to_save_trace)
            .expect("Failed to create a file to save call trace to");

        let filename = format!("{}.json", name);
        fs::write(dir_to_save_trace.join(filename), serialized_trace)
            .expect("Failed to write call trace to a file");
    }
}

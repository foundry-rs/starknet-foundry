use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::entry_point::{CallEntryPoint, CallType, ExecutionResources};
use cheatnet::constants::{TEST_CONTRACT_CLASS_HASH, TEST_ENTRY_POINT_SELECTOR};
use cheatnet::state::CallTrace;
use conversions::IntoConv;
use profiler::trace_data::{
    CallEntryPoint as ProfilerCallEntryPoint, CallTrace as ProfilerCallTrace,
    CallType as ProfilerCallType, DeprecatedSyscallSelector as ProfilerDeprecatedSyscallSelector,
    ExecutionResources as ProfilerExecutionResources, VmExecutionResources,
};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::ClassHash;
use starknet_api::hash::StarkFelt;
use starknet_api::stark_felt;

use crate::contracts_data::ContractsData;
use crate::test_case_summary::{Single, TestCaseSummary};

pub const TRACE_DIR: &str = ".snfoundry_trace";
pub const TEST_CODE_CONTRACT_NAME: &str = "SNFORGE_TEST_CODE";
pub const TEST_CODE_FUNCTION_NAME: &str = "SNFORGE_TEST_CODE_FUNCTION";

// #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
// pub enum ProfilerCallType {
//     #[default]
//     Call = 0,
//     Delegate = 1,
// }

// impl From<CallType> for ProfilerCallType {
//     fn from(value: CallType) -> Self {
//         match value {
//             CallType::Call => ProfilerCallType::Call,
//             CallType::Delegate => ProfilerCallType::Delegate,
//         }
//     }
// }

// #[derive(Debug, Default, Clone, Deserialize, Serialize)]
// pub struct ProfilerExecutionResources {
//     pub vm_resources: VmExecutionResources,
//     pub syscall_counter: ProfilerSyscallCounter,
// }

// impl AddAssign<&ProfilerExecutionResources> for ProfilerExecutionResources {
//     fn add_assign(&mut self, rhs: &ProfilerExecutionResources) {
//         self.vm_resources += &rhs.vm_resources;
//         for (syscall, count) in &rhs.syscall_counter {
//             *self.syscall_counter.entry(*syscall).or_insert(0) += count;
//         }
//     }
// }

// impl Sub<&ProfilerExecutionResources> for &ProfilerExecutionResources {
//     type Output = ProfilerExecutionResources;

//     fn sub(self, rhs: &ProfilerExecutionResources) -> Self::Output {
//         let mut result = self.clone();
//         result.vm_resources -= &rhs.vm_resources;
//         for (syscall, count) in &rhs.syscall_counter {
//             *result.syscall_counter.entry(*syscall).or_insert(0) -= count;
//         }
//         result
//     }
// }

#[must_use]
pub fn build_profiler_call_trace(
    value: CallTrace,
    contracts_data: &ContractsData,
) -> ProfilerCallTrace {
    ProfilerCallTrace {
        entry_point: build_profiler_call_entry_point(value.entry_point, contracts_data),
        cumulative_resources: build_profiler_execution_resources(value.used_execution_resources),
        nested_calls: value
            .nested_calls
            .into_iter()
            .map(|c| build_profiler_call_trace(c.borrow().clone(), contracts_data))
            .collect(),
    }
}

pub fn build_profiler_execution_resources(value: ExecutionResources) -> ProfilerExecutionResources {
    let mut syscall_counter = HashMap::new();
    for (key, val) in value.syscall_counter {
        syscall_counter.insert(build_profiler_deprecated_syscall_selector(key), val);
    }
    ProfilerExecutionResources {
        vm_resources: VmExecutionResources {
            n_steps: value.vm_resources.n_steps,
            n_memory_holes: value.vm_resources.n_memory_holes,
            builtin_instance_counter: value.vm_resources.builtin_instance_counter,
        },
        syscall_counter,
    }
}

#[must_use]
pub fn build_profiler_call_entry_point(
    value: CallEntryPoint,
    contracts_data: &ContractsData,
) -> ProfilerCallEntryPoint {
    let CallEntryPoint {
        class_hash,
        code_address,
        entry_point_type,
        entry_point_selector,
        calldata,
        storage_address,
        caller_address,
        call_type,
        initial_gas: _,
    } = value;

    let mut contract_name = class_hash
        .and_then(|c| contracts_data.class_hashes.get_by_right(&c))
        .cloned();
    let mut function_name = contracts_data.selectors.get(&entry_point_selector).cloned();

    if entry_point_selector.0
        == get_selector_from_name(TEST_ENTRY_POINT_SELECTOR)
            .unwrap()
            .into_()
        && class_hash == Some(ClassHash(stark_felt!(TEST_CONTRACT_CLASS_HASH)))
    {
        contract_name = Some(String::from(TEST_CODE_CONTRACT_NAME));
        function_name = Some(String::from(TEST_CODE_FUNCTION_NAME));
    }

    ProfilerCallEntryPoint {
        class_hash,
        code_address,
        entry_point_type,
        entry_point_selector,
        calldata,
        storage_address,
        caller_address,
        call_type: build_profiler_call_type(call_type),
        contract_name,
        function_name,
    }
}

fn build_profiler_deprecated_syscall_selector(
    value: DeprecatedSyscallSelector,
) -> ProfilerDeprecatedSyscallSelector {
    match value {
        DeprecatedSyscallSelector::CallContract => ProfilerDeprecatedSyscallSelector::CallContract,
        DeprecatedSyscallSelector::DelegateCall => ProfilerDeprecatedSyscallSelector::DelegateCall,
        DeprecatedSyscallSelector::DelegateL1Handler => {
            ProfilerDeprecatedSyscallSelector::DelegateL1Handler
        }
        DeprecatedSyscallSelector::Deploy => ProfilerDeprecatedSyscallSelector::Deploy,
        DeprecatedSyscallSelector::EmitEvent => ProfilerDeprecatedSyscallSelector::EmitEvent,
        DeprecatedSyscallSelector::GetBlockHash => ProfilerDeprecatedSyscallSelector::GetBlockHash,

        DeprecatedSyscallSelector::GetBlockNumber => {
            ProfilerDeprecatedSyscallSelector::GetBlockNumber
        }
        DeprecatedSyscallSelector::GetBlockTimestamp => {
            ProfilerDeprecatedSyscallSelector::GetBlockTimestamp
        }
        DeprecatedSyscallSelector::GetCallerAddress => {
            ProfilerDeprecatedSyscallSelector::GetCallerAddress
        }
        DeprecatedSyscallSelector::GetContractAddress => {
            ProfilerDeprecatedSyscallSelector::GetContractAddress
        }
        DeprecatedSyscallSelector::GetExecutionInfo => {
            ProfilerDeprecatedSyscallSelector::GetExecutionInfo
        }
        DeprecatedSyscallSelector::GetSequencerAddress => {
            ProfilerDeprecatedSyscallSelector::GetSequencerAddress
        }
        DeprecatedSyscallSelector::GetTxInfo => ProfilerDeprecatedSyscallSelector::GetTxInfo,
        DeprecatedSyscallSelector::GetTxSignature => {
            ProfilerDeprecatedSyscallSelector::GetTxSignature
        }
        DeprecatedSyscallSelector::Keccak => ProfilerDeprecatedSyscallSelector::Keccak,
        DeprecatedSyscallSelector::LibraryCall => ProfilerDeprecatedSyscallSelector::LibraryCall,
        DeprecatedSyscallSelector::LibraryCallL1Handler => {
            ProfilerDeprecatedSyscallSelector::LibraryCallL1Handler
        }
        DeprecatedSyscallSelector::ReplaceClass => ProfilerDeprecatedSyscallSelector::ReplaceClass,
        DeprecatedSyscallSelector::Secp256k1Add => ProfilerDeprecatedSyscallSelector::Secp256k1Add,
        DeprecatedSyscallSelector::Secp256k1GetPointFromX => {
            ProfilerDeprecatedSyscallSelector::Secp256k1GetPointFromX
        }
        DeprecatedSyscallSelector::Secp256k1GetXy => {
            ProfilerDeprecatedSyscallSelector::Secp256k1GetXy
        }
        DeprecatedSyscallSelector::Secp256k1Mul => ProfilerDeprecatedSyscallSelector::Secp256k1Mul,
        DeprecatedSyscallSelector::Secp256k1New => ProfilerDeprecatedSyscallSelector::Secp256k1New,
        DeprecatedSyscallSelector::Secp256r1Add => ProfilerDeprecatedSyscallSelector::Secp256r1Add,
        DeprecatedSyscallSelector::Secp256r1GetPointFromX => {
            ProfilerDeprecatedSyscallSelector::Secp256r1GetPointFromX
        }
        DeprecatedSyscallSelector::Secp256r1GetXy => {
            ProfilerDeprecatedSyscallSelector::Secp256r1GetXy
        }
        DeprecatedSyscallSelector::Secp256r1Mul => ProfilerDeprecatedSyscallSelector::Secp256r1Mul,
        DeprecatedSyscallSelector::Secp256r1New => ProfilerDeprecatedSyscallSelector::Secp256r1New,
        DeprecatedSyscallSelector::SendMessageToL1 => {
            ProfilerDeprecatedSyscallSelector::SendMessageToL1
        }
        DeprecatedSyscallSelector::StorageRead => ProfilerDeprecatedSyscallSelector::StorageRead,
        DeprecatedSyscallSelector::StorageWrite => ProfilerDeprecatedSyscallSelector::StorageWrite,
    }
}

fn build_profiler_call_type(value: CallType) -> ProfilerCallType {
    match value {
        CallType::Call => ProfilerCallType::Call,
        CallType::Delegate => ProfilerCallType::Delegate,
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

        let filename = format!("{name}.json");
        fs::write(dir_to_save_trace.join(filename), serialized_trace)
            .expect("Failed to write call trace to a file");
    }
}

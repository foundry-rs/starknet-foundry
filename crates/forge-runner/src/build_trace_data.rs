use anyhow::{Context, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::syscalls::hint_processor::SyscallCounter;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cairo_vm::vm::trace::trace_entry::TraceEntry;
use cheatnet::constants::{TEST_CONTRACT_CLASS_HASH, TEST_ENTRY_POINT_SELECTOR};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::{CallTrace, CallTraceNode};
use conversions::IntoConv;
use itertools::Itertools;
use starknet::core::utils::get_selector_from_name;
use starknet_api::class_hash;
use starknet_api::core::ClassHash;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkHash;
use trace_data::{
    CallEntryPoint as ProfilerCallEntryPoint, CallTrace as ProfilerCallTrace,
    CallTraceNode as ProfilerCallTraceNode, CallType as ProfilerCallType, ContractAddress,
    DeprecatedSyscallSelector as ProfilerDeprecatedSyscallSelector, EntryPointSelector,
    EntryPointType as ProfilerEntryPointType, ExecutionResources as ProfilerExecutionResources,
    TraceEntry as ProfilerTraceEntry, VmExecutionResources,
};

pub const TRACE_DIR: &str = ".snfoundry_trace";
pub const TEST_CODE_CONTRACT_NAME: &str = "SNFORGE_TEST_CODE";
pub const TEST_CODE_FUNCTION_NAME: &str = "SNFORGE_TEST_CODE_FUNCTION";

pub fn build_profiler_call_trace(
    value: &Rc<RefCell<CallTrace>>,
    contracts_data: &ContractsData,
) -> ProfilerCallTrace {
    let value = value.borrow();

    let vm_trace = value.vm_trace.as_ref().map(|trace_data| {
        trace_data
            .iter()
            .map(build_profiler_trace_entry)
            .collect_vec()
    });

    ProfilerCallTrace {
        entry_point: build_profiler_call_entry_point(value.entry_point.clone(), contracts_data),
        cumulative_resources: build_profiler_execution_resources(
            value.used_execution_resources.clone(),
            value.used_syscalls.clone(),
        ),
        used_l1_resources: value.used_l1_resources.clone(),
        nested_calls: value
            .nested_calls
            .iter()
            .map(|c| build_profiler_call_trace_node(c, contracts_data))
            .collect(),
        vm_trace,
    }
}

fn build_profiler_call_trace_node(
    value: &CallTraceNode,
    contracts_data: &ContractsData,
) -> ProfilerCallTraceNode {
    match value {
        CallTraceNode::EntryPointCall(trace) => {
            ProfilerCallTraceNode::EntryPointCall(build_profiler_call_trace(trace, contracts_data))
        }
        CallTraceNode::DeployWithoutConstructor => ProfilerCallTraceNode::DeployWithoutConstructor,
    }
}

#[must_use]
pub fn build_profiler_execution_resources(
    execution_resources: ExecutionResources,
    syscall_counter: SyscallCounter,
) -> ProfilerExecutionResources {
    let mut profiler_syscall_counter = HashMap::new();
    for (key, val) in syscall_counter {
        profiler_syscall_counter.insert(build_profiler_deprecated_syscall_selector(key), val);
    }
    ProfilerExecutionResources {
        vm_resources: VmExecutionResources {
            n_steps: execution_resources.n_steps,
            n_memory_holes: execution_resources.n_memory_holes,
            builtin_instance_counter: execution_resources.builtin_instance_counter,
        },
        syscall_counter: profiler_syscall_counter,
    }
}

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn build_profiler_call_entry_point(
    value: CallEntryPoint,
    contracts_data: &ContractsData,
) -> ProfilerCallEntryPoint {
    let CallEntryPoint {
        class_hash,
        entry_point_type,
        entry_point_selector,
        storage_address,
        call_type,
        ..
    } = value;

    let mut contract_name = class_hash
        .and_then(|c| contracts_data.class_hashes.get_by_right(&c))
        .cloned();
    let mut function_name = contracts_data.selectors.get(&entry_point_selector).cloned();

    if entry_point_selector.0
        == get_selector_from_name(TEST_ENTRY_POINT_SELECTOR)
            .unwrap()
            .into_()
        && class_hash == Some(class_hash!(TEST_CONTRACT_CLASS_HASH))
    {
        contract_name = Some(String::from(TEST_CODE_CONTRACT_NAME));
        function_name = Some(String::from(TEST_CODE_FUNCTION_NAME));
    }

    ProfilerCallEntryPoint {
        class_hash: class_hash.map(|ch| trace_data::ClassHash(ch.to_string())),
        entry_point_type: build_profiler_entry_point_type(entry_point_type),
        entry_point_selector: EntryPointSelector(format!("{}", entry_point_selector.0)),
        contract_address: ContractAddress(format!("{}", storage_address.0.key())),
        call_type: build_profiler_call_type(call_type),
        contract_name,
        function_name,
    }
}

fn build_profiler_entry_point_type(value: EntryPointType) -> ProfilerEntryPointType {
    match value {
        EntryPointType::Constructor => ProfilerEntryPointType::Constructor,
        EntryPointType::External => ProfilerEntryPointType::External,
        EntryPointType::L1Handler => ProfilerEntryPointType::L1Handler,
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

fn build_profiler_trace_entry(value: &TraceEntry) -> ProfilerTraceEntry {
    ProfilerTraceEntry {
        pc: value.pc,
        ap: value.ap,
        fp: value.fp,
    }
}

pub fn save_trace_data(test_name: &String, trace_data: &ProfilerCallTrace) -> Result<PathBuf> {
    let serialized_trace =
        serde_json::to_string(trace_data).expect("Failed to serialize call trace");
    let dir_to_save_trace = PathBuf::from(TRACE_DIR);
    fs::create_dir_all(&dir_to_save_trace).context("Failed to create a .trace_data directory")?;

    let filename = format!("{test_name}.json");
    fs::write(dir_to_save_trace.join(&filename), serialized_trace)
        .context("Failed to write call trace to a file")?;
    Ok(dir_to_save_trace.join(&filename))
}

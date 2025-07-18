use anyhow::{Context, Result};
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::syscalls::vm_syscall_utils::{
    SyscallSelector, SyscallUsage, SyscallUsageMap,
};

use cairo_annotations::trace_data::{
    CairoExecutionInfo, CallEntryPoint as ProfilerCallEntryPoint,
    CallTraceNode as ProfilerCallTraceNode, CallTraceV1 as ProfilerCallTrace,
    CallType as ProfilerCallType, CasmLevelInfo, ContractAddress,
    DeprecatedSyscallSelector as ProfilerDeprecatedSyscallSelector,
    EntryPointSelector as ProfilerEntryPointSelector, EntryPointType as ProfilerEntryPointType,
    ExecutionResources as ProfilerExecutionResources, SyscallUsage as ProfilerSyscallUsage,
    TraceEntry as ProfilerTraceEntry, VersionedCallTrace as VersionedProfilerCallTrace,
    VmExecutionResources,
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cairo_vm::vm::trace::trace_entry::RelocatedTraceEntry;
use camino::{Utf8Path, Utf8PathBuf};
use cheatnet::forking::data::ForkData;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::{CallTrace, CallTraceNode};
use conversions::IntoConv;
use conversions::string::TryFromHexStr;
use runtime::starknet::constants::{TEST_CONTRACT_CLASS_HASH, TEST_ENTRY_POINT_SELECTOR};
use starknet::core::utils::get_selector_from_name;
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::{ClassHash, EntryPointSelector};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub const TRACE_DIR: &str = "snfoundry_trace";

pub const TEST_CODE_CONTRACT_NAME: &str = "SNFORGE_TEST_CODE";
pub const TEST_CODE_FUNCTION_NAME: &str = "SNFORGE_TEST_CODE_FUNCTION";

pub fn build_profiler_call_trace(
    value: &Rc<RefCell<CallTrace>>,
    contracts_data: &ContractsData,
    fork_data: &ForkData,
    versioned_program_path: &Utf8Path,
) -> ProfilerCallTrace {
    let value = value.borrow();

    let entry_point =
        build_profiler_call_entry_point(value.entry_point.clone(), contracts_data, fork_data);
    let vm_trace = value
        .vm_trace
        .as_ref()
        .map(|trace_data| trace_data.iter().map(build_profiler_trace_entry).collect());
    let cairo_execution_info = build_cairo_execution_info(
        &value.entry_point,
        vm_trace,
        contracts_data,
        versioned_program_path,
    );

    ProfilerCallTrace {
        entry_point,
        cumulative_resources: build_profiler_execution_resources(
            value.used_execution_resources.clone(),
            value.get_total_used_syscalls(),
            value.gas_consumed,
        ),
        used_l1_resources: value.used_l1_resources.clone(),
        nested_calls: value
            .nested_calls
            .iter()
            .map(|c| {
                build_profiler_call_trace_node(c, contracts_data, fork_data, versioned_program_path)
            })
            .collect(),
        cairo_execution_info,
    }
}

fn build_cairo_execution_info(
    entry_point: &CallEntryPoint,
    vm_trace: Option<Vec<ProfilerTraceEntry>>,
    contracts_data: &ContractsData,
    versioned_program_path: &Utf8Path,
) -> Option<CairoExecutionInfo> {
    let contract_name = get_contract_name(entry_point.class_hash, contracts_data);
    let source_sierra_path = contract_name
        .and_then(|name| get_source_sierra_path(&name, contracts_data, versioned_program_path));

    Some(CairoExecutionInfo {
        casm_level_info: CasmLevelInfo {
            run_with_call_header: false,
            vm_trace: vm_trace?,
        },
        source_sierra_path: source_sierra_path?,
    })
}

fn get_source_sierra_path(
    contract_name: &str,
    contracts_data: &ContractsData,
    versioned_program_path: &Utf8Path,
) -> Option<Utf8PathBuf> {
    if contract_name == TEST_CODE_CONTRACT_NAME {
        Some(versioned_program_path.into())
    } else {
        contracts_data
            .get_source_sierra_path(contract_name)
            .cloned()
    }
}

fn build_profiler_call_trace_node(
    value: &CallTraceNode,
    contracts_data: &ContractsData,
    fork_data: &ForkData,
    versioned_program_path: &Utf8Path,
) -> ProfilerCallTraceNode {
    match value {
        CallTraceNode::EntryPointCall(trace) => ProfilerCallTraceNode::EntryPointCall(Box::new(
            build_profiler_call_trace(trace, contracts_data, fork_data, versioned_program_path),
        )),
        CallTraceNode::DeployWithoutConstructor => ProfilerCallTraceNode::DeployWithoutConstructor,
    }
}

#[must_use]
pub fn build_profiler_execution_resources(
    execution_resources: ExecutionResources,
    syscall_usage: SyscallUsageMap,
    gas_consumed: u64,
) -> ProfilerExecutionResources {
    let profiler_syscall_counter = syscall_usage
        .into_iter()
        .map(|(key, val)| {
            (
                build_profiler_deprecated_syscall_selector(key),
                build_profiler_syscall_usage(val),
            )
        })
        .collect();

    ProfilerExecutionResources {
        vm_resources: VmExecutionResources {
            n_steps: execution_resources.n_steps,
            n_memory_holes: execution_resources.n_memory_holes,
            builtin_instance_counter: execution_resources
                .builtin_instance_counter
                .into_iter()
                .map(|(key, value)| (key.to_str_with_suffix().to_owned(), value))
                .collect(),
        },
        gas_consumed: Some(gas_consumed),
        syscall_counter: Some(profiler_syscall_counter),
    }
}

#[must_use]
#[expect(clippy::needless_pass_by_value)]
pub fn build_profiler_call_entry_point(
    value: CallEntryPoint,
    contracts_data: &ContractsData,
    fork_data: &ForkData,
) -> ProfilerCallEntryPoint {
    let CallEntryPoint {
        class_hash,
        entry_point_type,
        entry_point_selector,
        storage_address,
        call_type,
        ..
    } = value;

    let contract_name = get_contract_name(class_hash, contracts_data);
    let function_name = get_function_name(&entry_point_selector, contracts_data, fork_data);

    ProfilerCallEntryPoint {
        class_hash: class_hash.map(|ch| cairo_annotations::trace_data::ClassHash(ch.0)),
        entry_point_type: build_profiler_entry_point_type(entry_point_type),
        entry_point_selector: ProfilerEntryPointSelector(entry_point_selector.0),
        contract_address: ContractAddress(*storage_address.0.key()),
        call_type: build_profiler_call_type(call_type),
        contract_name,
        function_name,
    }
}

fn get_contract_name(
    class_hash: Option<ClassHash>,
    contracts_data: &ContractsData,
) -> Option<String> {
    if class_hash == Some(TryFromHexStr::try_from_hex_str(TEST_CONTRACT_CLASS_HASH).unwrap()) {
        Some(String::from(TEST_CODE_CONTRACT_NAME))
    } else {
        class_hash
            .and_then(|c| contracts_data.get_contract_name(&c))
            .cloned()
    }
}

fn get_function_name(
    entry_point_selector: &EntryPointSelector,
    contracts_data: &ContractsData,
    fork_data: &ForkData,
) -> Option<String> {
    if entry_point_selector.0
        == get_selector_from_name(TEST_ENTRY_POINT_SELECTOR)
            .unwrap()
            .into_()
    {
        Some(TEST_CODE_FUNCTION_NAME.to_string())
    } else if let Some(name) = contracts_data
        .get_function_name(entry_point_selector)
        .cloned()
    {
        Some(name)
    } else {
        fork_data.selectors.get(entry_point_selector).cloned()
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
    value: SyscallSelector,
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
        DeprecatedSyscallSelector::Sha256ProcessBlock => {
            ProfilerDeprecatedSyscallSelector::Sha256ProcessBlock
        }
        DeprecatedSyscallSelector::GetClassHashAt => {
            ProfilerDeprecatedSyscallSelector::GetClassHashAt
        }
        DeprecatedSyscallSelector::KeccakRound => ProfilerDeprecatedSyscallSelector::KeccakRound,
        DeprecatedSyscallSelector::MetaTxV0 => ProfilerDeprecatedSyscallSelector::MetaTxV0,
    }
}

fn build_profiler_syscall_usage(
    SyscallUsage {
        call_count,
        linear_factor,
    }: SyscallUsage,
) -> ProfilerSyscallUsage {
    ProfilerSyscallUsage {
        call_count,
        linear_factor,
    }
}

fn build_profiler_call_type(value: CallType) -> ProfilerCallType {
    match value {
        CallType::Call => ProfilerCallType::Call,
        CallType::Delegate => ProfilerCallType::Delegate,
    }
}

fn build_profiler_trace_entry(value: &RelocatedTraceEntry) -> ProfilerTraceEntry {
    ProfilerTraceEntry {
        pc: value.pc,
        ap: value.ap,
        fp: value.fp,
    }
}

pub fn save_trace_data(
    test_name: &str,
    trace_data: &VersionedProfilerCallTrace,
) -> Result<PathBuf> {
    let serialized_trace =
        serde_json::to_string(trace_data).expect("Failed to serialize call trace");
    let dir_to_save_trace = PathBuf::from(TRACE_DIR);
    fs::create_dir_all(&dir_to_save_trace).context("Failed to create a .trace_data directory")?;

    let filename = format!("{test_name}.json");
    fs::write(dir_to_save_trace.join(&filename), serialized_trace)
        .context("Failed to write call trace to a file")?;
    Ok(dir_to_save_trace.join(&filename))
}

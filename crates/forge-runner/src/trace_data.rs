use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::ops::{AddAssign, Sub};
use std::path::PathBuf;
use std::rc::Rc;

// Will be provided by profiler crate in the future
// This module will be removed!
use blockifier::execution::entry_point::{CallEntryPoint, CallType, ExecutionResources};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources as VmExecutionResources;
use cheatnet::constants::{TEST_CONTRACT_CLASS_HASH, TEST_ENTRY_POINT_SELECTOR};
use cheatnet::state::{CallTrace, OnchainData};
use conversions::IntoConv;
use serde::{Deserialize, Serialize};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::stark_felt;
use starknet_api::transaction::Calldata;

use crate::contracts_data::ContractsData;
use crate::test_case_summary::{Single, TestCaseSummary};

pub const TRACE_DIR: &str = ".snfoundry_trace";
pub const TEST_CODE_CONTRACT_NAME: &str = "SNFORGE_TEST_CODE";
pub const TEST_CODE_FUNCTION_NAME: &str = "SNFORGE_TEST_CODE_FUNCTION";

/// Tree structure representing trace of a call.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfilerCallTrace {
    pub entry_point: ProfilerCallEntryPoint,
    // These also include resources used by internal calls
    pub used_execution_resources: ProfilerExecutionResources,
    pub used_onchain_data: OnchainData,
    pub nested_calls: Vec<ProfilerCallTrace>,
}

impl ProfilerCallTrace {
    pub fn from_call_trace(value: &Rc<RefCell<CallTrace>>, contracts_data: &ContractsData) -> Self {
        let value = value.borrow();

        ProfilerCallTrace {
            entry_point: ProfilerCallEntryPoint::from(value.entry_point.clone(), contracts_data),
            used_execution_resources: ProfilerExecutionResources::from(
                value.used_execution_resources.clone(),
            ),
            used_onchain_data: value.used_onchain_data,
            nested_calls: value
                .nested_calls
                .iter()
                .map(|c| ProfilerCallTrace::from_call_trace(c, contracts_data))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct ProfilerCallEntryPoint {
    pub class_hash: Option<ClassHash>,
    pub code_address: Option<ContractAddress>,
    pub entry_point_type: EntryPointType,
    pub entry_point_selector: EntryPointSelector,
    pub calldata: Calldata,
    pub storage_address: ContractAddress,
    pub caller_address: ContractAddress,
    pub call_type: ProfilerCallType,
    pub initial_gas: u64,

    /// Contract name to display instead of contract address
    pub contract_name: Option<String>,
    /// Function name to display instead of entry point selector
    pub function_name: Option<String>,
}

impl ProfilerCallEntryPoint {
    fn from(value: CallEntryPoint, contracts_data: &ContractsData) -> Self {
        let CallEntryPoint {
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
            call_type: call_type.into(),
            initial_gas,
            contract_name,
            function_name,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ProfilerCallType {
    #[default]
    Call = 0,
    Delegate = 1,
}

impl From<CallType> for ProfilerCallType {
    fn from(value: CallType) -> Self {
        match value {
            CallType::Call => ProfilerCallType::Call,
            CallType::Delegate => ProfilerCallType::Delegate,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ProfilerExecutionResources {
    pub vm_resources: VmExecutionResources,
    pub syscall_counter: ProfilerSyscallCounter,
}

impl AddAssign<&ProfilerExecutionResources> for ProfilerExecutionResources {
    fn add_assign(&mut self, rhs: &ProfilerExecutionResources) {
        self.vm_resources += &rhs.vm_resources;
        for (syscall, count) in &rhs.syscall_counter {
            *self.syscall_counter.entry(*syscall).or_insert(0) += count;
        }
    }
}

impl Sub<&ProfilerExecutionResources> for &ProfilerExecutionResources {
    type Output = ProfilerExecutionResources;

    fn sub(self, rhs: &ProfilerExecutionResources) -> Self::Output {
        let mut result = self.clone();
        result.vm_resources -= &rhs.vm_resources;
        for (syscall, count) in &rhs.syscall_counter {
            *result.syscall_counter.entry(*syscall).or_insert(0) -= count;
        }
        result
    }
}

impl ProfilerExecutionResources {
    #[must_use]
    pub fn gt_eq_than(&self, other: &ProfilerExecutionResources) -> bool {
        if self.vm_resources.n_steps < other.vm_resources.n_steps
            || self.vm_resources.n_memory_holes < other.vm_resources.n_memory_holes
        {
            return false;
        }

        let self_builtin_counter = &self.vm_resources.builtin_instance_counter;
        let other_builtin_counter = &other.vm_resources.builtin_instance_counter;
        for (builtin, other_count) in other_builtin_counter {
            let self_count = self_builtin_counter.get(builtin).unwrap_or(&0);
            if self_count < other_count {
                return false;
            }
        }

        let self_builtin_counter = &self.syscall_counter;
        let other_builtin_counter = &other.syscall_counter;
        for (syscall, other_count) in other_builtin_counter {
            let self_count = self_builtin_counter.get(syscall).unwrap_or(&0);
            if self_count < other_count {
                return false;
            }
        }

        true
    }
}

impl From<ExecutionResources> for ProfilerExecutionResources {
    fn from(value: ExecutionResources) -> Self {
        let mut syscall_counter = HashMap::new();
        for (key, val) in value.syscall_counter {
            syscall_counter.insert(key.into(), val);
        }
        ProfilerExecutionResources {
            vm_resources: value.vm_resources,
            syscall_counter,
        }
    }
}

type ProfilerSyscallCounter = HashMap<ProfilerDeprecatedSyscallSelector, usize>;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Hash, PartialEq)]
pub enum ProfilerDeprecatedSyscallSelector {
    CallContract,
    DelegateCall,
    DelegateL1Handler,
    Deploy,
    EmitEvent,
    GetBlockHash,
    GetBlockNumber,
    GetBlockTimestamp,
    GetCallerAddress,
    GetContractAddress,
    GetExecutionInfo,
    GetSequencerAddress,
    GetTxInfo,
    GetTxSignature,
    Keccak,
    LibraryCall,
    LibraryCallL1Handler,
    ReplaceClass,
    Secp256k1Add,
    Secp256k1GetPointFromX,
    Secp256k1GetXy,
    Secp256k1Mul,
    Secp256k1New,
    Secp256r1Add,
    Secp256r1GetPointFromX,
    Secp256r1GetXy,
    Secp256r1Mul,
    Secp256r1New,
    SendMessageToL1,
    StorageRead,
    StorageWrite,
}

impl From<DeprecatedSyscallSelector> for ProfilerDeprecatedSyscallSelector {
    fn from(value: DeprecatedSyscallSelector) -> Self {
        match value {
            DeprecatedSyscallSelector::CallContract => {
                ProfilerDeprecatedSyscallSelector::CallContract
            }
            DeprecatedSyscallSelector::DelegateCall => {
                ProfilerDeprecatedSyscallSelector::DelegateCall
            }
            DeprecatedSyscallSelector::DelegateL1Handler => {
                ProfilerDeprecatedSyscallSelector::DelegateL1Handler
            }
            DeprecatedSyscallSelector::Deploy => ProfilerDeprecatedSyscallSelector::Deploy,
            DeprecatedSyscallSelector::EmitEvent => ProfilerDeprecatedSyscallSelector::EmitEvent,
            DeprecatedSyscallSelector::GetBlockHash => {
                ProfilerDeprecatedSyscallSelector::GetBlockHash
            }

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
            DeprecatedSyscallSelector::LibraryCall => {
                ProfilerDeprecatedSyscallSelector::LibraryCall
            }
            DeprecatedSyscallSelector::LibraryCallL1Handler => {
                ProfilerDeprecatedSyscallSelector::LibraryCallL1Handler
            }
            DeprecatedSyscallSelector::ReplaceClass => {
                ProfilerDeprecatedSyscallSelector::ReplaceClass
            }
            DeprecatedSyscallSelector::Secp256k1Add => {
                ProfilerDeprecatedSyscallSelector::Secp256k1Add
            }
            DeprecatedSyscallSelector::Secp256k1GetPointFromX => {
                ProfilerDeprecatedSyscallSelector::Secp256k1GetPointFromX
            }
            DeprecatedSyscallSelector::Secp256k1GetXy => {
                ProfilerDeprecatedSyscallSelector::Secp256k1GetXy
            }
            DeprecatedSyscallSelector::Secp256k1Mul => {
                ProfilerDeprecatedSyscallSelector::Secp256k1Mul
            }
            DeprecatedSyscallSelector::Secp256k1New => {
                ProfilerDeprecatedSyscallSelector::Secp256k1New
            }
            DeprecatedSyscallSelector::Secp256r1Add => {
                ProfilerDeprecatedSyscallSelector::Secp256r1Add
            }
            DeprecatedSyscallSelector::Secp256r1GetPointFromX => {
                ProfilerDeprecatedSyscallSelector::Secp256r1GetPointFromX
            }
            DeprecatedSyscallSelector::Secp256r1GetXy => {
                ProfilerDeprecatedSyscallSelector::Secp256r1GetXy
            }
            DeprecatedSyscallSelector::Secp256r1Mul => {
                ProfilerDeprecatedSyscallSelector::Secp256r1Mul
            }
            DeprecatedSyscallSelector::Secp256r1New => {
                ProfilerDeprecatedSyscallSelector::Secp256r1New
            }
            DeprecatedSyscallSelector::SendMessageToL1 => {
                ProfilerDeprecatedSyscallSelector::SendMessageToL1
            }
            DeprecatedSyscallSelector::StorageRead => {
                ProfilerDeprecatedSyscallSelector::StorageRead
            }
            DeprecatedSyscallSelector::StorageWrite => {
                ProfilerDeprecatedSyscallSelector::StorageWrite
            }
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

        let filename = format!("{name}.json");
        fs::write(dir_to_save_trace.join(filename), serialized_trace)
            .expect("Failed to write call trace to a file");
    }
}

use blockifier::execution::deprecated_syscalls::hint_processor::SyscallCounter;
use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_lang_runner::casm_run::format_next_item;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::execute_call_entry_point;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::panic_data::try_extract_panic_data;
use crate::runtime_extensions::common::{create_entry_point_selector, create_execute_calldata};
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::entry_point::EntryPointExecutionResult;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::execution::{
    entry_point::{CallEntryPoint, CallType, ExecutionResources},
    errors::{EntryPointExecutionError, PreExecutionError},
};
use blockifier::state::errors::StateError;
use cairo_felt::Felt252;
use starknet_api::core::ClassHash;
use starknet_api::{core::ContractAddress, deprecated_contract_class::EntryPointType};

use super::RuntimeState;

#[derive(Clone, Debug, Default)]
pub struct UsedResources {
    pub execution_resources: ExecutionResources,
    pub l2_to_l1_payloads_length: Vec<usize>,
}

impl UsedResources {
    pub fn extend(self: &mut UsedResources, other: &UsedResources) {
        self.execution_resources.vm_resources += &other.execution_resources.vm_resources;

        self.update_syscall_counter(&other.execution_resources.syscall_counter);

        self.l2_to_l1_payloads_length
            .extend(&other.l2_to_l1_payloads_length);
    }

    fn update_syscall_counter(self: &mut UsedResources, syscall_counter: &SyscallCounter) {
        for (syscall, count) in syscall_counter {
            *self
                .execution_resources
                .syscall_counter
                .entry(*syscall)
                .or_insert(0) += count;
        }
    }
}

pub(crate) fn subtract_syscall_counters(
    syscall_counter: &SyscallCounter,
    subtrahend: &SyscallCounter,
) -> SyscallCounter {
    let mut result = syscall_counter.clone();

    for (syscall, count) in subtrahend {
        let old_syscall_count = syscall_counter
            .get(syscall)
            .unwrap_or_else(|| panic!("Missing SyscallCounter entry {syscall:?}"));

        let new_count = old_syscall_count
            .checked_sub(*count)
            .unwrap_or_else(|| panic!("Underflow when subtracting syscall counts for {syscall:?}"));

        if new_count != 0 {
            result.insert(*syscall, new_count);
        } else {
            result.remove(syscall);
        }
    }

    result
}

/// Enum representing possible call execution result, along with the data
#[derive(Debug)]
pub enum CallResult {
    Success { ret_data: Vec<Felt252> },
    Failure(CallFailure),
}

/// Enum representing possible call failure and its' type.
/// `Panic` - Recoverable, meant to be caught by the user.
/// `Error` - Unrecoverable, equivalent of panic! in rust.
#[derive(Debug)]
pub enum CallFailure {
    Panic { panic_data: Vec<Felt252> },
    Error { msg: String },
}

pub enum AddressOrClassHash {
    ContractAddress(ContractAddress),
    ClassHash(ClassHash),
}

impl CallFailure {
    /// Maps blockifier-type error, to one that can be put into memory as panic-data (or re-raised)
    #[must_use]
    pub fn from_execution_error(
        err: &EntryPointExecutionError,
        starknet_identifier: &AddressOrClassHash,
    ) -> Self {
        match err {
            EntryPointExecutionError::ExecutionFailed { error_data } => {
                let err_data: Vec<Felt252> = error_data
                    .iter()
                    .map(|data| Felt252::from_bytes_be(data.bytes()))
                    .collect();

                // blockifier/src/execution_utils:274 (format_panic_data) (modified)
                let err_data_str = {
                    let mut felts = error_data.iter().map(|felt| stark_felt_to_felt(*felt));
                    let mut items = Vec::new();
                    while let Some(item) = format_next_item(&mut felts) {
                        items.push(item.quote_if_string());
                    }
                    if let [item] = &items[..] {
                        item.clone()
                    } else {
                        items.join("\n").to_string()
                    }
                };

                for invalid_calldata_msg in [
                    "Failed to deserialize param #",
                    "Input too long for arguments",
                ] {
                    if err_data_str.contains(invalid_calldata_msg) {
                        return CallFailure::Error { msg: err_data_str };
                    }
                }

                CallFailure::Panic {
                    panic_data: err_data,
                }
            }
            EntryPointExecutionError::VirtualMachineExecutionErrorWithTrace { trace, .. } => {
                if let Some(panic_data) = try_extract_panic_data(trace) {
                    CallFailure::Panic { panic_data }
                } else {
                    CallFailure::Error { msg: trace.clone() }
                }
            }
            EntryPointExecutionError::PreExecutionError(PreExecutionError::EntryPointNotFound(
                selector,
            )) => {
                let selector_hash = selector.0;
                let msg = match starknet_identifier {
                    AddressOrClassHash::ContractAddress(address) => format!(
                        "Entry point selector {selector_hash} not found in contract {}",
                        address.0.key()
                    ),
                    AddressOrClassHash::ClassHash(class_hash) => format!(
                        "Entry point selector {selector_hash} not found for class hash {class_hash}"
                    ),
                };
                CallFailure::Error { msg }
            }
            EntryPointExecutionError::PreExecutionError(
                PreExecutionError::UninitializedStorageAddress(contract_address),
            ) => {
                let address = contract_address.0.key().to_string();
                let msg = format!("Contract not deployed at address: {address}");
                CallFailure::Error { msg }
            }
            EntryPointExecutionError::StateError(StateError::StateReadError(msg)) => {
                CallFailure::Error { msg: msg.clone() }
            }
            result => CallFailure::Error {
                msg: result.to_string(),
            },
        }
    }
}

impl CallResult {
    fn from_execution_result(
        result: &EntryPointExecutionResult<CallInfo>,
        starknet_identifier: &AddressOrClassHash,
    ) -> Self {
        match result {
            Ok(call_info) => {
                let raw_return_data = &call_info.execution.retdata.0;

                let return_data = raw_return_data
                    .iter()
                    .map(|data| Felt252::from_bytes_be(data.bytes()))
                    .collect();

                CallResult::Success {
                    ret_data: return_data,
                }
            }
            Err(err) => {
                CallResult::Failure(CallFailure::from_execution_error(err, starknet_identifier))
            }
        }
    }
}

pub fn call_l1_handler(
    syscall_handler: &mut SyscallHintProcessor,
    runtime_state: &mut RuntimeState,
    contract_address: &ContractAddress,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
) -> CallResult {
    let entry_point_selector = create_entry_point_selector(entry_point_selector);
    let calldata = create_execute_calldata(calldata);

    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(*contract_address),
        entry_point_type: EntryPointType::L1Handler,
        entry_point_selector,
        calldata,
        storage_address: *contract_address,
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    call_entry_point(
        syscall_handler,
        runtime_state,
        entry_point,
        &AddressOrClassHash::ContractAddress(*contract_address),
    )
}

pub fn call_entry_point(
    syscall_handler: &mut SyscallHintProcessor,
    runtime_state: &mut RuntimeState,
    mut entry_point: CallEntryPoint,
    starknet_identifier: &AddressOrClassHash,
) -> CallResult {
    let exec_result = execute_call_entry_point(
        &mut entry_point,
        syscall_handler.state,
        runtime_state,
        syscall_handler.resources,
        syscall_handler.context,
    );

    let result = CallResult::from_execution_result(&exec_result, starknet_identifier);

    if let Ok(call_info) = exec_result {
        syscall_handler.inner_calls.push(call_info);
    };

    result
}

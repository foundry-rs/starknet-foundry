use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::constants::TEST_ADDRESS;
use crate::panic_data::try_extract_panic_data;
use crate::state::BlockifierState;
use crate::{
    constants::{build_block_context, build_transaction_context},
    execution::{entry_point::execute_call_entry_point, gas::gas_from_execution_resources},
    CheatnetState,
};
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::entry_point::EntryPointExecutionResult;
use blockifier::execution::{
    entry_point::{CallEntryPoint, CallType, EntryPointExecutionContext, ExecutionResources},
    errors::{EntryPointExecutionError, PreExecutionError},
};
use blockifier::state::errors::StateError;
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use starknet_api::core::PatriciaKey;
use starknet_api::patricia_key;
use starknet_api::{
    core::{ContractAddress, EntryPointSelector},
    deprecated_contract_class::EntryPointType,
    hash::{StarkFelt, StarkHash},
    transaction::Calldata,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceReport {
    pub gas: f64,
    pub steps: usize,
    pub bultins: HashMap<String, usize>,
}

impl ResourceReport {
    pub(crate) fn new(gas: f64, resources: &ExecutionResources) -> Self {
        Self {
            gas,
            steps: resources.vm_resources.n_steps,
            bultins: resources.vm_resources.builtin_instance_counter.clone(),
        }
    }
}

/// Represents contract output, along with the data and the resources consumed during execution
#[derive(Debug)]
pub struct CallContractOutput {
    pub result: CallContractResult,
    pub resource_report: ResourceReport,
}

/// Enum representing possible contract execution result, along with the data
#[derive(Debug)]
pub enum CallContractResult {
    Success { ret_data: Vec<Felt252> },
    Failure(CallContractFailure),
}

/// Enum representing possible call failure and its' type.
/// `Panic` - Recoverable, meant to be caught by the user.
/// `Error` - Unrecoverable, equivalent of panic! in rust.
#[derive(Debug)]
pub enum CallContractFailure {
    Panic { panic_data: Vec<Felt252> },
    Error { msg: String },
}

impl CallContractFailure {
    /// Maps blockifier-type error, to one that can be put into memory as panic-data (or re-raised)
    #[must_use]
    pub fn from_execution_error(
        err: &EntryPointExecutionError,
        contract_address: &ContractAddress,
    ) -> Self {
        match err {
            EntryPointExecutionError::ExecutionFailed { error_data } => {
                let err_data: Vec<Felt252> = error_data
                    .iter()
                    .map(|data| Felt252::from_bytes_be(data.bytes()))
                    .collect();

                let err_data_str = err_data
                    .iter()
                    .map(|x| as_cairo_short_string(x).unwrap())
                    .collect::<Vec<String>>()
                    .join("\n");

                for invalid_calldata_msg in [
                    "Failed to deserialize param #",
                    "Input too long for arguments",
                ] {
                    if err_data_str.contains(invalid_calldata_msg) {
                        return CallContractFailure::Error { msg: err_data_str };
                    }
                }

                CallContractFailure::Panic {
                    panic_data: err_data,
                }
            }
            EntryPointExecutionError::VirtualMachineExecutionErrorWithTrace { trace, .. } => {
                if let Some(panic_data) = try_extract_panic_data(trace) {
                    CallContractFailure::Panic { panic_data }
                } else {
                    CallContractFailure::Error { msg: trace.clone() }
                }
            }
            EntryPointExecutionError::PreExecutionError(PreExecutionError::EntryPointNotFound(
                selector,
            )) => {
                let selector_hash = selector.0;
                let contract_addr = contract_address.0.key();
                let msg = format!(
                    "Entry point selector {selector_hash} not found in contract {contract_addr}"
                );
                CallContractFailure::Error { msg }
            }
            EntryPointExecutionError::PreExecutionError(
                PreExecutionError::UninitializedStorageAddress(contract_address),
            ) => {
                let address = contract_address.0.key().to_string();
                let msg = format!("Contract not deployed at address: {address}");
                CallContractFailure::Error { msg }
            }
            EntryPointExecutionError::StateError(StateError::StateReadError(msg)) => {
                CallContractFailure::Error { msg: msg.clone() }
            }
            result => CallContractFailure::Error {
                msg: result.to_string(),
            },
        }
    }
}

impl CallContractResult {
    fn from_execution_result(
        result: &EntryPointExecutionResult<CallInfo>,
        contract_address: &ContractAddress,
    ) -> Self {
        match result {
            Ok(call_info) => {
                let raw_return_data = &call_info.execution.retdata.0;

                let return_data = raw_return_data
                    .iter()
                    .map(|data| Felt252::from_bytes_be(data.bytes()))
                    .collect();

                CallContractResult::Success {
                    ret_data: return_data,
                }
            }
            Err(err) => CallContractResult::Failure(CallContractFailure::from_execution_error(
                err,
                contract_address,
            )),
        }
    }
}

// This does contract call without the transaction layer. This way `call_contract` can return data and modify state.
// `call` and `invoke` on the transactional layer use such method under the hood.
pub fn call_contract(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
) -> Result<CallContractOutput> {
    call_entry_point(
        blockifier_state,
        cheatnet_state,
        contract_address,
        entry_point_selector,
        calldata,
        EntryPointType::External,
    )
}

pub fn call_l1_handler(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
) -> Result<CallContractOutput> {
    call_entry_point(
        blockifier_state,
        cheatnet_state,
        contract_address,
        entry_point_selector,
        calldata,
        EntryPointType::L1Handler,
    )
}

#[allow(clippy::too_many_lines)]
pub fn call_entry_point(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
    entry_point_type: EntryPointType,
) -> Result<CallContractOutput> {
    let entry_point_selector =
        EntryPointSelector(StarkHash::new(entry_point_selector.to_be_bytes())?);

    let calldata = Calldata(Arc::new(
        calldata
            .iter()
            .map(|data| StarkFelt::new(data.to_be_bytes()))
            .collect::<Result<Vec<_>, _>>()?,
    ));
    let mut entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(*contract_address),
        entry_point_type,
        entry_point_selector,
        calldata,
        storage_address: *contract_address,
        // test_contract address
        caller_address: ContractAddress(patricia_key!(TEST_ADDRESS)),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    let mut resources = ExecutionResources::default();
    let account_context = build_transaction_context();
    let block_context = build_block_context(cheatnet_state.block_info);

    let mut context = EntryPointExecutionContext::new(
        block_context.clone(),
        account_context,
        block_context.invoke_tx_max_n_steps.try_into().unwrap(),
    );

    let exec_result = execute_call_entry_point(
        &mut entry_point,
        blockifier_state.blockifier_state,
        cheatnet_state,
        &mut resources,
        &mut context,
    );

    let gas = gas_from_execution_resources(&block_context, &resources);
    let resource_report = ResourceReport::new(gas, &resources);
    let result = CallContractResult::from_execution_result(&exec_result, contract_address);

    Ok(CallContractOutput {
        result,
        resource_report,
    })
}

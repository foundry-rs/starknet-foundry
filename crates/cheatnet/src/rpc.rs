use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::panic_data::try_extract_panic_data;
use crate::state::BlockifierState;
use crate::{
    constants::{build_block_context, build_transaction_context},
    execution::{entry_point::execute_call_entry_point, gas::gas_from_execution_resources},
    CheatnetState,
};
use blockifier::execution::{
    entry_point::{CallEntryPoint, CallType, EntryPointExecutionContext, ExecutionResources},
    errors::{EntryPointExecutionError, PreExecutionError},
};
use blockifier::state::errors::StateError;
use cairo_felt::Felt252;
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
    fn new(gas: f64, resources: &ExecutionResources) -> Self {
        Self {
            gas,
            steps: resources.vm_resources.n_steps,
            bultins: resources.vm_resources.builtin_instance_counter.clone(),
        }
    }
}

#[derive(Debug)]
pub enum CallContractOutput {
    Success {
        ret_data: Vec<Felt252>,
        resource_report: ResourceReport,
    },
    Panic {
        panic_data: Vec<Felt252>,
        resource_report: ResourceReport,
    },
    Error {
        msg: String,
        resource_report: ResourceReport,
    },
}

// This does contract call without the transaction layer. This way `call_contract` can return data and modify state.
// `call` and `invoke` on the transactional layer use such method under the hood.
#[allow(clippy::too_many_lines)]
pub fn call_contract(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
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
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata,
        storage_address: *contract_address,
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    let mut resources = ExecutionResources::default();
    let account_context = build_transaction_context();
    let block_context = build_block_context();

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

    match exec_result {
        Ok(call_info) => {
            let raw_return_data = &call_info.execution.retdata.0;

            let return_data = raw_return_data
                .iter()
                .map(|data| Felt252::from_bytes_be(data.bytes()))
                .collect();

            Ok(CallContractOutput::Success {
                ret_data: return_data,
                resource_report,
            })
        }
        Err(EntryPointExecutionError::ExecutionFailed { error_data }) => {
            let err_data = error_data
                .iter()
                .map(|data| Felt252::from_bytes_be(data.bytes()))
                .collect();

            Ok(CallContractOutput::Panic {
                panic_data: err_data,
                resource_report,
            })
        }
        Err(EntryPointExecutionError::VirtualMachineExecutionErrorWithTrace { trace, .. }) => {
            if let Some(panic_data) = try_extract_panic_data(&trace) {
                Ok(CallContractOutput::Panic {
                    panic_data,
                    resource_report,
                })
            } else {
                Ok(CallContractOutput::Error {
                    msg: trace,
                    resource_report,
                })
            }
        }
        Err(EntryPointExecutionError::PreExecutionError(
            PreExecutionError::EntryPointNotFound(selector),
        )) => {
            let selector_hash = selector.0;
            let contract_addr = contract_address.0.key();
            let msg = format!(
                "Entry point selector {selector_hash} not found in contract {contract_addr}"
            );
            Ok(CallContractOutput::Error {
                msg,
                resource_report,
            })
        }
        Err(EntryPointExecutionError::PreExecutionError(
            PreExecutionError::UninitializedStorageAddress(contract_address),
        )) => {
            let address = contract_address.0.key().to_string();
            let msg = format!("Contract not deployed at address: {address}");
            Ok(CallContractOutput::Error {
                msg,
                resource_report,
            })
        }
        Err(EntryPointExecutionError::StateError(StateError::StateReadError(msg))) => {
            Ok(CallContractOutput::Error {
                msg,
                resource_report,
            })
        }
        result => panic!("Unparseable result: {result:?}"),
    }
}

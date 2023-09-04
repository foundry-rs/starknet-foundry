use anyhow::Result;
// <<<<<<< HEAD
// use cairo_vm::{
//     hint_processor::hint_processor_definition::{HintProcessor, HintReference},
//     serde::deserialize_program::ApTracking,
//     types::{
//         exec_scope::ExecutionScopes,
//         relocatable::{MaybeRelocatable, Relocatable},
//     },
//     vm::{
//         errors::{hint_errors::HintError, vm_errors::VirtualMachineError},
//         runners::cairo_runner::{CairoArg, CairoRunner, RunResources},
//         vm_core::VirtualMachine,
//     },
// };
// use std::collections::HashSet;
// use std::{any::Any, collections::HashMap, sync::Arc};
//
// use crate::{
//     constants::{build_block_context, build_transaction_context},
//     CheatnetState,
// };
// use blockifier::execution::call_info::{CallExecution, CallInfo, OrderedEvent, Retdata};
// use blockifier::execution::entry_point::{handle_empty_constructor, ConstructorContext};
// use blockifier::{
//     abi::constants,
//     execution::{
//         execution_utils::ReadOnlySegment,
//         syscalls::{
//             hint_processor::{create_retdata_segment, write_segment, OUT_OF_GAS_ERROR},
//             CallContractRequest, WriteResponseResult,
//         },
//     },
//     transaction::transaction_utils::update_remaining_gas,
// };
// use blockifier::{
//     execution::{
//         cairo1_execution::{
//             finalize_execution, initialize_execution_context, prepare_call_arguments,
//             VmExecutionContext,
//         },
//         common_hints::HintExecutionResult,
//         contract_class::{ContractClass, ContractClassV1, EntryPointV1},
//         deprecated_syscalls::DeprecatedSyscallSelector,
//         entry_point::{
//             CallEntryPoint, CallType, EntryPointExecutionContext, EntryPointExecutionResult,
//             ExecutionResources, FAULTY_CLASS_HASH,
//         },
//         errors::{EntryPointExecutionError, PreExecutionError, VirtualMachineExecutionError},
//         execution_utils::{felt_to_stark_felt, stark_felt_to_felt, Args},
//         syscalls::{
//             hint_processor::{SyscallExecutionError, SyscallHintProcessor},
//             EmptyRequest, GetExecutionInfoResponse,
//         },
//     },
//     state::state_api::State,
// };
// use cairo_felt::Felt252;
// use cairo_lang_casm::{
//     hints::{Hint, StarknetHint},
//     operand::{BinOpOperand, DerefOrImmediate, Operation, Register, ResOperand},
// };
// use cairo_vm::vm::runners::cairo_runner::ExecutionResources as VmExecutionResources;
// use starknet_api::{
//     core::{ClassHash, ContractAddress, EntryPointSelector},
//     deprecated_contract_class::EntryPointType,
//     hash::{StarkFelt, StarkHash},
//     transaction::{Calldata, TransactionVersion},
// };
//
// use crate::cheatcodes::spy_events::Event;
// use blockifier::execution::syscalls::{
//     DeployRequest, DeployResponse, LibraryCallRequest, SyscallRequest, SyscallRequestWrapper,
//     SyscallResponse, SyscallResponseWrapper, SyscallResult,
// };
// use blockifier::state::errors::StateError;
// use cairo_vm::hint_processor::hint_processor_definition::HintProcessorLogic;
// use cairo_vm::vm::runners::cairo_runner::ResourceTracker;
// use starknet_api::core::calculate_contract_address;
// =======
use std::sync::Arc;
// >>>>>>> master

use crate::panic_data::try_extract_panic_data;
use crate::{
    constants::{build_block_context, build_transaction_context},
    execution::{
        entry_point::execute_call_entry_point, events::collect_emitted_events_from_spied_contracts,
    },
    CheatnetState,
};
use blockifier::execution::{
    entry_point::{CallEntryPoint, CallType, EntryPointExecutionContext, ExecutionResources},
    errors::{EntryPointExecutionError, PreExecutionError},
};
use cairo_felt::Felt252;
use starknet_api::{
    core::{ContractAddress, EntryPointSelector},
    deprecated_contract_class::EntryPointType,
    hash::{StarkFelt, StarkHash},
    transaction::Calldata,
};

pub enum CallContractOutput {
    Success { ret_data: Vec<Felt252> },
    Panic { panic_data: Vec<Felt252> },
    Error { msg: String },
}

// This does contract call without the transaction layer. This way `call_contract` can return data and modify state.
// `call` and `invoke` on the transactional layer use such method under the hood.
pub fn call_contract(
    contract_address: &ContractAddress,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
    cheatnet_state: &mut CheatnetState,
) -> Result<CallContractOutput> {
    let blockifier_state = &mut cheatnet_state.blockifier_state;
    let cheatcode_state = &mut cheatnet_state.cheatcode_state;

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
        blockifier_state,
        cheatcode_state,
        &mut resources,
        &mut context,
    );

    match exec_result {
        Ok(call_info) => {
            if !cheatcode_state.spies.is_empty() {
                let mut events =
                    collect_emitted_events_from_spied_contracts(&call_info, cheatcode_state);
                cheatcode_state.detected_events.append(&mut events);
            }

            let raw_return_data = &call_info.execution.retdata.0;

            let return_data = raw_return_data
                .iter()
                .map(|data| Felt252::from_bytes_be(data.bytes()))
                .collect();

            Ok(CallContractOutput::Success {
                ret_data: return_data,
            })
        }
        Err(EntryPointExecutionError::ExecutionFailed { error_data }) => {
            let err_data = error_data
                .iter()
                .map(|data| Felt252::from_bytes_be(data.bytes()))
                .collect();

            Ok(CallContractOutput::Panic {
                panic_data: err_data,
            })
        }
        Err(EntryPointExecutionError::VirtualMachineExecutionErrorWithTrace { trace, .. }) => {
            if let Some(panic_data) = try_extract_panic_data(&trace) {
                Ok(CallContractOutput::Panic { panic_data })
            } else {
                Ok(CallContractOutput::Error { msg: trace })
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
            Ok(CallContractOutput::Error { msg })
        }
        Err(EntryPointExecutionError::PreExecutionError(
            PreExecutionError::UninitializedStorageAddress(contract_address),
        )) => {
            let address = contract_address.0.key().to_string();
            let msg = format!("Contract not deployed at address: {address}");
            Ok(CallContractOutput::Error { msg })
        }
        result => panic!("Unparseable result: {result:?}"),
    }
}

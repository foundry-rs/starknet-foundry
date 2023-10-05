use crate::constants::{build_block_context, build_transaction_context};
use crate::state::BlockifierState;
use crate::CheatnetState;
use anyhow::Result;
use blockifier::abi::constants;
use blockifier::execution::entry_point::{
    ConstructorContext, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::felt_to_stark_felt;
use std::sync::Arc;

use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use conversions::StarknetConversions;

use crate::execution::syscalls::execute_deployment;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::transaction::Calldata;

use super::CheatcodeError;
use crate::rpc::{CallContractFailure, ResourceReport};

#[allow(clippy::module_name_repetitions)]
pub struct DeployCallPayload {
    pub resource_report: ResourceReport,
    pub contract_address: ContractAddress,
}

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::cast_precision_loss)]
pub fn deploy_at(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    calldata: &[Felt252],
    contract_address: ContractAddress,
) -> Result<DeployCallPayload, CheatcodeError> {
    cheatnet_state.increment_deploy_salt_base();
    let blockifier_state_raw: &mut dyn State = blockifier_state.blockifier_state;

    let entry_point_execution_ctx = &mut EntryPointExecutionContext::new(
        build_block_context(),
        build_transaction_context(),
        constants::MAX_STEPS_PER_TX,
    );

    let ctor_context = ConstructorContext {
        class_hash: *class_hash,
        code_address: Some(contract_address),
        storage_address: contract_address,
        caller_address: Felt252::from(0x0000_1724_9872_3497_3219_3472_1083_7402_i128)
            .to_contract_address(), // TODO: Extract to consts
    };

    let calldata = Calldata(Arc::new(
        calldata.to_vec().iter().map(felt_to_stark_felt).collect(),
    ));

    execute_deployment(
        blockifier_state_raw,
        &mut ExecutionResources::default(),
        entry_point_execution_ctx,
        ctor_context,
        calldata,
        u64::MAX,
        cheatnet_state,
    )
    .map(|call_info| DeployCallPayload {
        resource_report: ResourceReport {
            gas: call_info.execution.gas_consumed as f64,
            steps: call_info.vm_resources.n_steps,
            bultins: call_info.vm_resources.builtin_instance_counter,
        },
        contract_address,
    })
    .map_err(|err| {
        let call_contract_failure =
            CallContractFailure::from_execution_error(&err, &contract_address);
        CheatcodeError::from(call_contract_failure)
    })

    // TODO: assess if we even need this
    // match call_result {
    //     CallContractOutput::Success {
    //         resource_report, ..
    //     } => match result {
    //         Ok(contract_address) => Ok(DeployPayload {
    //             contract_address,
    //             resource_report,
    //         }),
    //         Err(cheatcode_error) => Err(cheatcode_error),
    //     },
    //     CallContractOutput::Panic { panic_data, .. } => {
    //         let panic_data_str = panic_data
    //             .iter()
    //             .map(|x| as_cairo_short_string(x).unwrap())
    //             .collect::<Vec<String>>()
    //             .join("\n");
    //
    //         for invalid_calldata_msg in [
    //             "Failed to deserialize param #",
    //             "Input too long for arguments",
    //         ] {
    //             if panic_data_str.contains(invalid_calldata_msg) {
    //                 return Err(CheatcodeError::Unrecoverable(EnhancedHintError::from(
    //                     CustomHint(Box::from(panic_data_str)),
    //                 )));
    //             }
    //         }
    //
    //         Err(CheatcodeError::Recoverable(panic_data))
    //     }
    //     CallContractOutput::Error { msg, .. } => Err(CheatcodeError::Unrecoverable(
    //         EnhancedHintError::from(CustomHint(Box::from(msg))),
    //     )),
    // }
}

#[allow(clippy::module_name_repetitions)]
pub fn deploy(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    calldata: &[Felt252],
) -> Result<DeployCallPayload, CheatcodeError> {
    let contract_address = cheatnet_state.precalculate_address(class_hash, calldata);

    deploy_at(
        blockifier_state,
        cheatnet_state,
        class_hash,
        calldata,
        contract_address,
    )
}

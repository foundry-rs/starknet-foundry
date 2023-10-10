use crate::constants as crate_constants;
use crate::constants::{build_block_context, build_transaction_context};
use crate::state::BlockifierState;
use crate::CheatnetState;
use anyhow::Result;
use blockifier::abi::constants as blockifier_constants;
use blockifier::execution::entry_point::{
    ConstructorContext, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::felt_to_stark_felt;
use std::sync::Arc;

use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use conversions::StarknetConversions;
use num_traits::ToPrimitive;

use crate::cheatcodes::EnhancedHintError;
use crate::execution::syscalls::execute_deployment;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::transaction::Calldata;

use super::CheatcodeError;
use crate::rpc::{CallContractFailure, ResourceReport};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct DeployCallPayload {
    pub resource_report: ResourceReport,
    pub contract_address: ContractAddress,
}

#[allow(clippy::module_name_repetitions)]
pub fn deploy_at(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    calldata: &[Felt252],
    contract_address: ContractAddress,
) -> Result<DeployCallPayload, CheatcodeError> {
    let blockifier_state_raw: &mut dyn State = blockifier_state.blockifier_state;

    if let Ok(class_hash) = blockifier_state_raw.get_class_hash_at(contract_address) {
        if class_hash != ClassHash::default() {
            return Err(CheatcodeError::Unrecoverable(EnhancedHintError::from(
                CustomHint(Box::from("Address is already taken")),
            )));
        }
    }

    let entry_point_execution_ctx = &mut EntryPointExecutionContext::new(
        build_block_context(),
        build_transaction_context(),
        blockifier_constants::MAX_STEPS_PER_TX,
    );

    let ctor_context = ConstructorContext {
        class_hash: *class_hash,
        code_address: Some(contract_address),
        storage_address: contract_address,
        caller_address: crate_constants::TEST_ADDRESS.to_contract_address(),
    };

    let calldata = Calldata(Arc::new(
        calldata.to_vec().iter().map(felt_to_stark_felt).collect(),
    ));

    let resources = &mut ExecutionResources::default();

    let result = execute_deployment(
        blockifier_state_raw,
        resources,
        entry_point_execution_ctx,
        ctor_context,
        calldata,
        u64::MAX,
        cheatnet_state,
    )
    .map(|call_info| DeployCallPayload {
        resource_report: ResourceReport::new(
            call_info
                .execution
                .gas_consumed
                .to_f64()
                .expect("Conversion to f64 failed")
                + blockifier_constants::DEPLOY_GAS_COST
                    .to_f64()
                    .expect("Conversion to f64 failed"),
            resources,
        ),
        contract_address,
    })
    .map_err(|err| {
        let call_contract_failure =
            CallContractFailure::from_execution_error(&err, &contract_address);
        CheatcodeError::from(call_contract_failure)
    });
    cheatnet_state.increment_deploy_salt_base();
    result
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

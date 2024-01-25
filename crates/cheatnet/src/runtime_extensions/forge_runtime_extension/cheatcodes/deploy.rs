use crate::constants::TEST_ADDRESS;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    AddressOrClassHash, CallFailure,
};
use crate::state::BlockifierState;
use crate::CheatnetState;
use anyhow::Result;
use blockifier::execution::common_hints::ExecutionMode;
use blockifier::execution::entry_point::{
    ConstructorContext, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::felt_to_stark_felt;
use runtime::starknet::context::{build_block_context, build_transaction_context};
use runtime::EnhancedHintError;
use std::sync::Arc;

use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use starknet_api::core::PatriciaKey;
use starknet_api::hash::StarkHash;
use starknet_api::patricia_key;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::cheated_syscalls;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::transaction::Calldata;

use super::CheatcodeError;

#[derive(Debug)]
pub struct DeployCallPayload {
    pub used_resources: ExecutionResources,
    pub contract_address: ContractAddress,
}

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
        &build_block_context(cheatnet_state.block_info),
        &build_transaction_context(),
        ExecutionMode::Execute,
        false,
    )
    .unwrap();

    let ctor_context = ConstructorContext {
        class_hash: *class_hash,
        code_address: Some(contract_address),
        storage_address: contract_address,
        caller_address: ContractAddress(patricia_key!(TEST_ADDRESS)),
    };

    let calldata = Calldata(Arc::new(
        calldata.to_vec().iter().map(felt_to_stark_felt).collect(),
    ));

    let resources = &mut ExecutionResources::default();
    let result = cheated_syscalls::execute_deployment(
        blockifier_state_raw,
        resources,
        entry_point_execution_ctx,
        ctor_context,
        calldata,
        u64::MAX,
        cheatnet_state,
    )
    .map(|_call_info| DeployCallPayload {
        used_resources: resources.clone(),
        contract_address,
    })
    .map_err(|err| {
        let call_contract_failure = CallFailure::from_execution_error(
            &err,
            &AddressOrClassHash::ContractAddress(contract_address),
        );
        CheatcodeError::from(call_contract_failure)
    });
    cheatnet_state.increment_deploy_salt_base();
    result
}

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

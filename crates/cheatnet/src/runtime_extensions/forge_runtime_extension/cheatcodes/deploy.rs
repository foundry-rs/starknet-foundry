use crate::constants::TEST_ADDRESS;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    AddressOrClassHash, CallFailure,
};
use anyhow::Result;
use blockifier::execution::entry_point::ConstructorContext;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use runtime::EnhancedHintError;
use std::sync::Arc;

use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use starknet_types_core::felt::Felt;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::cheated_syscalls;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::transaction::Calldata;

use super::CheatcodeError;
use crate::state::CheatnetState;
use conversions::string::TryFromHexStr;

pub fn deploy_at(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    calldata: &[Felt],
    contract_address: ContractAddress,
    ignore_resources: bool,
) -> Result<(ContractAddress, Vec<Felt>), CheatcodeError> {
    if let Ok(class_hash) = syscall_handler.state.get_class_hash_at(contract_address) {
        if class_hash != ClassHash::default() {
            return Err(CheatcodeError::Unrecoverable(EnhancedHintError::from(
                CustomHint(Box::from("Address is already taken")),
            )));
        }
    }

    let ctor_context = ConstructorContext {
        class_hash: *class_hash,
        code_address: Some(contract_address),
        storage_address: contract_address,
        caller_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
    };

    let calldata = Calldata(Arc::new(calldata.to_vec()));

    // Remember the number of steps and memory holes before the deployment
    let (prev_n_steps, prev_n_memory_holes) = ignore_resources
        .then_some((
            syscall_handler.resources.n_steps,
            syscall_handler.resources.n_memory_holes,
        ))
        .unzip();

    let exec_result = cheated_syscalls::execute_deployment(
        syscall_handler.state,
        cheatnet_state,
        syscall_handler.resources,
        syscall_handler.context,
        &ctor_context,
        calldata,
        u64::MAX,
    );

    // Reset the number of steps and memory holes after the deployment
    if let (Some(steps), Some(memory_holes)) = (prev_n_steps, prev_n_memory_holes) {
        syscall_handler.resources.n_steps = steps;
        syscall_handler.resources.n_memory_holes = memory_holes;
    }

    cheatnet_state.increment_deploy_salt_base();

    match exec_result {
        Ok(call_info) => {
            let retdata = call_info.execution.retdata.0.clone();
            if !ignore_resources {
                syscall_handler.inner_calls.push(call_info);
            }
            Ok((contract_address, retdata))
        }
        Err(err) => {
            let call_contract_failure = CallFailure::from_execution_error(
                &err,
                &AddressOrClassHash::ContractAddress(contract_address),
            );
            Err(CheatcodeError::from(call_contract_failure))
        }
    }
}

pub fn deploy(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    calldata: &[Felt],
) -> Result<(ContractAddress, Vec<Felt>), CheatcodeError> {
    let contract_address = cheatnet_state.precalculate_address(class_hash, calldata);

    deploy_at(
        syscall_handler,
        cheatnet_state,
        class_hash,
        calldata,
        contract_address,
        false,
    )
}

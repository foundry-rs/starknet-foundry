use crate::constants::TEST_ADDRESS;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    AddressOrClassHash, CallFailure,
};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::RuntimeState;
use anyhow::Result;
use blockifier::execution::entry_point::ConstructorContext;
use blockifier::execution::execution_utils::felt_to_stark_felt;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use conversions::FromConv;
use runtime::EnhancedHintError;
use std::sync::Arc;

use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use starknet_api::core::PatriciaKey;
use starknet_api::hash::StarkHash;
use starknet_api::{contract_address, patricia_key};

use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::cheated_syscalls;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::transaction::Calldata;

use super::CheatcodeError;

pub fn deploy_at(
    syscall_handler: &mut SyscallHintProcessor,
    runtime_state: &mut RuntimeState,
    class_hash: &ClassHash,
    calldata: &[Felt252],
    contract_address: ContractAddress,
) -> Result<(ContractAddress, Vec<Felt252>), CheatcodeError> {
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
        caller_address: contract_address!(TEST_ADDRESS),
    };

    let calldata = Calldata(Arc::new(
        calldata.to_vec().iter().map(felt_to_stark_felt).collect(),
    ));

    let exec_result = cheated_syscalls::execute_deployment(
        syscall_handler.state,
        runtime_state,
        syscall_handler.resources,
        syscall_handler.context,
        ctor_context,
        calldata,
        u64::MAX,
    );
    runtime_state.cheatnet_state.increment_deploy_salt_base();

    match exec_result {
        Ok(call_info) => {
            let retdata = call_info.execution.retdata.0.clone();
            let retdata: Vec<Felt252> = retdata.into_iter().map(Felt252::from_).collect();
            syscall_handler.inner_calls.push(call_info);
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
    runtime_state: &mut RuntimeState,
    class_hash: &ClassHash,
    calldata: &[Felt252],
) -> Result<(ContractAddress, Vec<Felt252>), CheatcodeError> {
    let contract_address = runtime_state
        .cheatnet_state
        .precalculate_address(class_hash, calldata);

    deploy_at(
        syscall_handler,
        runtime_state,
        class_hash,
        calldata,
        contract_address,
    )
}

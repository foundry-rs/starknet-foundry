use anyhow::Result;
use std::sync::Arc;

use blockifier::{state::{cached_state::CachedState, state_api::State}, execution::{entry_point::{CallEntryPoint, CallType, ExecutionResources, EntryPointExecutionContext, EntryPointExecutionResult, CallInfo, FAULTY_CLASS_HASH}, errors::{EntryPointExecutionError, PreExecutionError}, contract_class::{ContractClassV1, ContractClass}, cairo1_execution::{VmExecutionContext, initialize_execution_context, prepare_call_arguments, finalize_execution, run_entry_point}}};
use cairo_felt_blockifier::Felt252;
use starknet_api::{core::{ContractAddress, PatriciaKey, EntryPointSelector, ClassHash}, hash::{StarkFelt, StarkHash}, patricia_key, transaction::{Calldata, TransactionVersion}, deprecated_contract_class::EntryPointType};

use crate::{state::DictStateReader, constants::{TEST_ACCOUNT_CONTRACT_ADDRESS, build_transaction_context, build_block_context}};


// This can mutate state, the name of the syscall is not very good
pub fn call_contract(
    contract_address: &Felt252,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
    blockifier_state: &mut CachedState<DictStateReader>,
) -> Result<Vec<Felt252>> {
    let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
        contract_address.to_be_bytes(),
    )?)?);
    let entry_point_selector =
        EntryPointSelector(StarkHash::new(entry_point_selector.to_be_bytes())?);
    let account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
    let calldata = Calldata(Arc::new(
        calldata
            .iter()
            .map(|data| StarkFelt::new(data.to_be_bytes()))
            .collect::<Result<Vec<_>, _>>()?,
    ));
    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(contract_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata,
        storage_address: contract_address,
        caller_address: account_address,
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    let mut resources = ExecutionResources::default();
    let account_context = build_transaction_context();
    let block_context = build_block_context();

    let mut context = EntryPointExecutionContext::new(
        block_context.clone(),
        account_context,
        block_context.invoke_tx_max_n_steps,
    );

    let call_info = entry_point
        .execute(blockifier_state, &mut resources, &mut context)
        .unwrap();

    let raw_return_data = &call_info.execution.retdata.0;
    assert!(!call_info.execution.failed);

    let return_data = raw_return_data
        .iter()
        .map(|data| Felt252::from_bytes_be(data.bytes()))
        .collect();

    Ok(return_data)
}

pub fn execute_call_entry_point(
    entry_point: CallEntryPoint,
    state: &mut dyn State,
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> EntryPointExecutionResult<CallInfo> {
    // TODO it is private but unnecessary
    // context.current_recursion_depth += 1;
    // if context.current_recursion_depth > context.max_recursion_depth {
    //     return Err(EntryPointExecutionError::RecursionDepthExceeded);
    // }

    // Validate contract is deployed.
    let storage_address = entry_point.storage_address;
    let storage_class_hash = state.get_class_hash_at(entry_point.storage_address)?;
    if storage_class_hash == ClassHash::default() {
        return Err(PreExecutionError::UninitializedStorageAddress(entry_point.storage_address).into());
    }

    let class_hash = match entry_point.class_hash {
        Some(class_hash) => class_hash,
        None => storage_class_hash, // If not given, take the storage contract class hash.
    };
    // Hack to prevent version 0 attack on argent accounts.
    if context.account_tx_context.version == TransactionVersion(StarkFelt::from(0_u8))
        && class_hash
            == ClassHash(
                StarkFelt::try_from(FAULTY_CLASS_HASH).expect("A class hash must be a felt."),
            )
    {
        return Err(PreExecutionError::FraudAttempt.into());
    }
    // Add class hash to the call, that will appear in the output (call info).
    // entry_point.class_hash = Some(class_hash); // TODO
    let contract_class = state.get_compiled_contract_class(&class_hash)?;
    let result = execute_if_cairo1(entry_point, contract_class, state, resources, context)
        .map_err(|error| {
            match error {
                // On VM error, pack the stack trace into the propagated error.
                EntryPointExecutionError::VirtualMachineExecutionError(error) => {
                    context.error_stack.push((storage_address, error.try_to_vm_trace()));
                    // TODO(Dori, 1/5/2023): Call error_trace only in the top call; as it is
                    // right now,  each intermediate VM error is wrapped
                    // in a VirtualMachineExecutionErrorWithTrace  error
                    // with the stringified trace of all errors below
                    // it.
                    EntryPointExecutionError::VirtualMachineExecutionErrorWithTrace {
                        trace: context.error_trace(),
                        source: error,
                    }
                }
                other_error => other_error,
            }
        });

    // context.current_recursion_depth -= 1; // TODO
    result
}

pub fn execute_if_cairo1(
    call: CallEntryPoint,
    contract_class: ContractClass,
    state: &mut dyn State,
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> EntryPointExecutionResult<CallInfo> {
    match contract_class {
        ContractClass::V0(_) => todo!(),
        ContractClass::V1(contract_class) => execute_entry_point_call_cairo1(
            call,
            contract_class,
            state,
            resources,
            context,
        ),
    }
}

/// Executes a specific call to a contract entry point and returns its output.
pub fn execute_entry_point_call_cairo1(
    call: CallEntryPoint,
    contract_class: ContractClassV1,
    state: &mut dyn State,
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> EntryPointExecutionResult<CallInfo> {
    let VmExecutionContext {
        mut runner,
        mut vm,
        mut syscall_handler,
        initial_syscall_ptr,
        entry_point,
        program_segment_size,
    } = initialize_execution_context(call, &contract_class, state, resources, context)?;

    let args = prepare_call_arguments(
        &syscall_handler.call,
        &mut vm,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
        &entry_point,
    )?;
    let n_total_args = args.len();

    // Fix the VM resources, in order to calculate the usage of this run at the end.
    let previous_vm_resources = syscall_handler.resources.vm_resources.clone();

    // Execute.
    run_entry_point(
        &mut vm,
        &mut runner,
        &mut syscall_handler,
        entry_point,
        args,
        program_segment_size,
    )?;

    let call_info =
        finalize_execution(vm, runner, syscall_handler, previous_vm_resources, n_total_args)?;
    if call_info.execution.failed {
        return Err(EntryPointExecutionError::ExecutionFailed {
            error_data: call_info.execution.retdata.0,
        });
    }

    Ok(call_info)
}
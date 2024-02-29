use super::cairo1_execution::execute_entry_point_call_cairo1;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::deprecated::cairo0_execution::execute_entry_point_call_cairo0;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::RuntimeState;
use crate::state::CheatnetState;
use blockifier::execution::call_info::{CallExecution, Retdata};
use blockifier::{
    execution::{
        call_info::CallInfo,
        contract_class::ContractClass,
        entry_point::{
            handle_empty_constructor, CallEntryPoint, CallType, ConstructorContext,
            EntryPointExecutionContext, EntryPointExecutionResult, ExecutionResources,
            FAULTY_CLASS_HASH,
        },
        errors::{EntryPointExecutionError, PreExecutionError},
    },
    state::state_api::State,
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources as VmExecutionResources;
use starknet_api::{
    core::ClassHash,
    deprecated_contract_class::EntryPointType,
    hash::StarkFelt,
    transaction::{Calldata, TransactionVersion},
};
use std::collections::HashSet;
use cairo_felt::Felt252;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{AddressOrClassHash, CallResult};

// blockifier/src/execution/entry_point.rs:180 (CallEntryPoint::execute)
pub fn execute_call_entry_point(
    entry_point: &mut CallEntryPoint, // Instead of 'self'
    state: &mut dyn State,
    runtime_state: &mut RuntimeState,
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> EntryPointExecutionResult<CallInfo> {
    let cheated_data = if let CallType::Delegate = entry_point.call_type {
        runtime_state
            .cheatnet_state
            .trace_data
            .current_call_stack
            .top_cheated_data()
            .clone()
    } else {
        let contract_address = &entry_point.storage_address;
        let cheated_data_ = runtime_state
            .cheatnet_state
            .create_cheated_data(contract_address);
        runtime_state.cheatnet_state.update_cheats(contract_address);
        cheated_data_
    };

    // region: Modified blockifier code
    // We skip recursion depth validation here.
    runtime_state.cheatnet_state.trace_data.enter_nested_call(
        entry_point.clone(),
        resources.clone(),
        cheated_data,
    );

    if let Some(ret_data) =
        get_ret_data_by_call_entry_point(entry_point, runtime_state.cheatnet_state)
    {
        runtime_state.cheatnet_state.trace_data.exit_nested_call(
            resources,
            CallResult::Success {
                ret_data: ret_data
                    .iter()
                    .map(|data| Felt252::from_bytes_be(data.bytes()))
                    .collect(),
            },
        );
        return Ok(mocked_call_info(entry_point.clone(), ret_data));
    }
    // endregion

    // Validate contract is deployed.
    let storage_address = entry_point.storage_address;
    let storage_class_hash = state.get_class_hash_at(entry_point.storage_address)?;
    if storage_class_hash == ClassHash::default() {
        return Err(
            PreExecutionError::UninitializedStorageAddress(entry_point.storage_address).into(),
        );
    }
    let class_hash = runtime_state
        .cheatnet_state
        .replaced_bytecode_contracts
        .get(&storage_address)
        .copied()
        .or(entry_point.class_hash)
        .unwrap_or(storage_class_hash);
    // let class_hash = match entry_point.class_hash {
    //     Some(class_hash) => class_hash,
    //     None => storage_class_hash, // If not given, take the storage contract class hash.
    // };

    // region: Modified blockifier code
    runtime_state
        .cheatnet_state
        .trace_data
        .set_class_hash_for_current_call(class_hash);
    // endregion

    // Hack to prevent version 0 attack on argent accounts.
    if context.account_tx_context.version() == TransactionVersion(StarkFelt::from(0_u8))
        && class_hash
            == ClassHash(
                StarkFelt::try_from(FAULTY_CLASS_HASH).expect("A class hash must be a felt."),
            )
    {
        return Err(PreExecutionError::FraudAttempt.into());
    }
    // Add class hash to the call, that will appear in the output (call info).
    entry_point.class_hash = Some(class_hash);
    let contract_class = state.get_compiled_contract_class(&class_hash)?;

    // Region: Modified blockifier code
    let result = match contract_class {
        ContractClass::V0(deprecated_class) => execute_entry_point_call_cairo0(
            entry_point.clone(),
            deprecated_class,
            state,
            runtime_state,
            resources,
            context,
        ),
        ContractClass::V1(contract_class) => execute_entry_point_call_cairo1(
            entry_point.clone(),
            &contract_class,
            state,
            runtime_state,
            resources,
            context,
        ),
    };

    let result = result.map_err(|error| {
        // endregion
        match error {
            // On VM error, pack the stack trace into the propagated error.
            EntryPointExecutionError::VirtualMachineExecutionError(error) => {
                context
                    .error_stack
                    .push((storage_address, error.try_to_vm_trace()));
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

    let identifier = match entry_point.call_type {
        CallType::Call => AddressOrClassHash::ContractAddress(entry_point.storage_address),
        CallType::Delegate => AddressOrClassHash::ClassHash(entry_point.class_hash.unwrap()),
    };
    runtime_state.cheatnet_state.trace_data.exit_nested_call(
        resources,
        CallResult::from_execution_result(&result, &identifier),
    );

    result
    // region: Modified blockifier code
    // We skip recursion depth decrease
    // endregion
}

// blockifier/src/execution/entry_point.rs:366 (execute_constructor_entry_point)
pub fn execute_constructor_entry_point(
    state: &mut dyn State,
    runtime_state: &mut RuntimeState,
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
    ctor_context: ConstructorContext,
    calldata: Calldata,
    remaining_gas: u64,
) -> EntryPointExecutionResult<CallInfo> {
    // Ensure the class is declared (by reading it).
    let contract_class = state.get_compiled_contract_class(&ctor_context.class_hash)?;
    let Some(constructor_selector) = contract_class.constructor_selector() else {
        // Contract has no constructor.
        return handle_empty_constructor(ctor_context, calldata, remaining_gas);
    };

    let mut constructor_call = CallEntryPoint {
        class_hash: None,
        code_address: ctor_context.code_address,
        entry_point_type: EntryPointType::Constructor,
        entry_point_selector: constructor_selector,
        calldata,
        storage_address: ctor_context.storage_address,
        caller_address: ctor_context.caller_address,
        call_type: CallType::Call,
        initial_gas: remaining_gas,
    };
    // region: Modified blockifier code
    execute_call_entry_point(
        &mut constructor_call,
        state,
        runtime_state,
        resources,
        context,
    )
    // endregion
}

fn get_ret_data_by_call_entry_point(
    call: &CallEntryPoint,
    cheatnet_state: &CheatnetState,
) -> Option<Vec<StarkFelt>> {
    if let Some(contract_address) = call.code_address {
        if let Some(contract_functions) = cheatnet_state.mocked_functions.get(&contract_address) {
            let entrypoint_selector = call.entry_point_selector;

            let ret_data = contract_functions
                .get(&entrypoint_selector)
                .map(Clone::clone);
            return ret_data;
        }
    }
    None
}

fn mocked_call_info(call: CallEntryPoint, ret_data: Vec<StarkFelt>) -> CallInfo {
    CallInfo {
        call,
        execution: CallExecution {
            retdata: Retdata(ret_data),
            events: vec![],
            l2_to_l1_messages: vec![],
            failed: false,
            gas_consumed: 0,
        },
        vm_resources: VmExecutionResources::default(),
        inner_calls: vec![],
        storage_read_values: vec![],
        accessed_storage_keys: HashSet::new(),
    }
}

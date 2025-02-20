use super::cairo1_execution::execute_entry_point_call_cairo1;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::deprecated::cairo0_execution::execute_entry_point_call_cairo0;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{AddressOrClassHash, CallResult};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::common::{get_relocated_vm_trace, sum_syscall_counters};
use crate::state::{CallTrace, CallTraceNode, CheatStatus, EncounteredError};
use blockifier::execution::call_info::{CallExecution, Retdata};
use blockifier::execution::contract_class::RunnableCompiledClass;
use blockifier::execution::deprecated_syscalls::hint_processor::SyscallCounter;
use blockifier::{
    execution::{
        call_info::CallInfo,
        entry_point::{
            handle_empty_constructor, CallEntryPoint, CallType, ConstructorContext,
            EntryPointExecutionContext, EntryPointExecutionResult,
            FAULTY_CLASS_HASH,
        },
        errors::{EntryPointExecutionError, PreExecutionError},
    },
    state::state_api::State,
};
use cairo_vm::vm::runners::cairo_runner::{CairoRunner, ExecutionResources};
use cairo_vm::vm::trace::trace_entry::RelocatedTraceEntry;
use conversions::string::TryFromHexStr;
use starknet_api::{
    contract_class::EntryPointType,
    core::ClassHash,
    transaction::{fields::Calldata, TransactionVersion},
};
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use thiserror::Error;
use conversions::FromConv;

pub(crate) type ContractClassEntryPointExecutionResult = Result<
    (CallInfo, SyscallCounter, Option<Vec<RelocatedTraceEntry>>),
    EntryPointExecutionErrorWithTrace,
>;

// blockifier/src/execution/entry_point.rs:180 (CallEntryPoint::execute)
#[allow(clippy::too_many_lines)]
pub fn execute_call_entry_point(
    entry_point: &mut CallEntryPoint, // Instead of 'self'
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
) -> EntryPointExecutionResult<CallInfo> {
    let cheated_data = if let CallType::Delegate = entry_point.call_type {
        cheatnet_state
            .trace_data
            .current_call_stack
            .top_cheated_data()
            .clone()
    } else {
        let contract_address = entry_point.storage_address;
        let cheated_data_ = cheatnet_state.create_cheated_data(contract_address);
        cheatnet_state.update_cheats(&contract_address);
        cheated_data_
    };

    // region: Modified blockifier code
    // We skip recursion depth validation here.
    cheatnet_state
        .trace_data
        .enter_nested_call(entry_point.clone(), cheated_data);

    if let Some(cheat_status) = get_mocked_function_cheat_status(entry_point, cheatnet_state) {
        if let CheatStatus::Cheated(ret_data, _) = (*cheat_status).clone() {
            cheat_status.decrement_cheat_span();
            let ret_data_f252: Vec<Felt> =
                ret_data.iter().map(|datum| Felt::from_(*datum)).collect();
            cheatnet_state.trace_data.exit_nested_call(
                ExecutionResources::default(),
                Default::default(),
                CallResult::Success {
                    ret_data: ret_data_f252,
                },
                &[],
                None,
            );
            return Ok(mocked_call_info(entry_point.clone(), ret_data.clone()));
        }
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
    let maybe_replacement_class = cheatnet_state
        .replaced_bytecode_contracts
        .get(&storage_address)
        .copied();

    let class_hash = entry_point
        .class_hash
        .or(maybe_replacement_class)
        .unwrap_or(storage_class_hash); // If not given, take the storage contract class hash.

    // region: Modified blockifier code
    cheatnet_state
        .trace_data
        .set_class_hash_for_current_call(class_hash);
    // endregion

    // Hack to prevent version 0 attack on argent accounts.
    if context.tx_context.tx_info.version() == TransactionVersion(Felt::from(0_u8))
        && class_hash
            == TryFromHexStr::try_from_hex_str(FAULTY_CLASS_HASH)
                .expect("A class hash must be a felt.")
    {
        return Err(PreExecutionError::FraudAttempt.into());
    }
    // Add class hash to the call, that will appear in the output (call info).
    entry_point.class_hash = Some(class_hash);
    let contract_class = state.get_compiled_class(class_hash)?;

    // Region: Modified blockifier code
    let result = match contract_class {
        RunnableCompiledClass::V0(compiled_class_v0) => execute_entry_point_call_cairo0(
            entry_point.clone(),
            compiled_class_v0,
            state,
            cheatnet_state,
            context,
        ),
        RunnableCompiledClass::V1(compiled_class_v1) => execute_entry_point_call_cairo1(
            entry_point.clone(),
            &compiled_class_v1,
            state,
            cheatnet_state,
            context,
        ),
    };

    // region: Modified blockifier code
    match result {
        Ok((call_info, syscall_counter, vm_trace)) => {
            remove_syscall_resources_and_exit_success_call(
                &call_info,
                &syscall_counter,
                context,
                cheatnet_state,
                vm_trace,
            );
            Ok(call_info)
        }
        Err(EntryPointExecutionErrorWithTrace { source: err, trace }) => {
            if let Some(pc) = trace
                .as_ref()
                .and_then(|trace| trace.last())
                .map(|entry| entry.pc)
            {
                cheatnet_state
                    .encountered_errors
                    .push(EncounteredError { pc, class_hash });
            }
            exit_error_call(&err, cheatnet_state, entry_point, trace);
            Err(err)
        }
    }
    // endregion
}

fn remove_syscall_resources_and_exit_success_call(
    call_info: &CallInfo,
    syscall_counter: &SyscallCounter,
    context: &mut EntryPointExecutionContext,
    cheatnet_state: &mut CheatnetState,
    vm_trace: Option<Vec<RelocatedTraceEntry>>,
) {
    let versioned_constants = context.tx_context.block_context.versioned_constants();
    // We don't want the syscall resources to pollute the results
    let mut resources = call_info.resources.clone();
    resources -= &versioned_constants.get_additional_os_syscall_resources(syscall_counter);

    let nested_syscall_counter_sum =
        aggregate_nested_syscall_counters(&cheatnet_state.trace_data.current_call_stack.top());
    let syscall_counter = sum_syscall_counters(nested_syscall_counter_sum, syscall_counter);
    cheatnet_state.trace_data.exit_nested_call(
        resources,
        syscall_counter,
        CallResult::from_success(call_info),
        &call_info.execution.l2_to_l1_messages,
        vm_trace,
    );
}

fn exit_error_call(
    error: &EntryPointExecutionError,
    cheatnet_state: &mut CheatnetState,
    entry_point: &CallEntryPoint,
    vm_trace: Option<Vec<RelocatedTraceEntry>>,
) {
    let identifier = match entry_point.call_type {
        CallType::Call => AddressOrClassHash::ContractAddress(entry_point.storage_address),
        CallType::Delegate => AddressOrClassHash::ClassHash(entry_point.class_hash.unwrap()),
    };
    cheatnet_state.trace_data.exit_nested_call(
        ExecutionResources::default(),
        Default::default(),
        CallResult::from_err(error, &identifier),
        &[],
        vm_trace,
    );
}

// blockifier/src/execution/entry_point.rs:366 (execute_constructor_entry_point)
pub fn execute_constructor_entry_point(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    ctor_context: &ConstructorContext,
    calldata: Calldata,
    remaining_gas: u64,
) -> EntryPointExecutionResult<CallInfo> {
    // Ensure the class is declared (by reading it).
    let contract_class = state.get_compiled_class(ctor_context.class_hash)?;
    let Some(constructor_selector) = contract_class.constructor_selector() else {
        // Contract has no constructor.
        cheatnet_state
            .trace_data
            .add_deploy_without_constructor_node();
        return handle_empty_constructor(
            contract_class,
            context,
            ctor_context,
            calldata,
            remaining_gas,
        );
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
    execute_call_entry_point(&mut constructor_call, state, cheatnet_state, context)
    // endregion
}

fn get_mocked_function_cheat_status<'a>(
    call: &CallEntryPoint,
    cheatnet_state: &'a mut CheatnetState,
) -> Option<&'a mut CheatStatus<Vec<Felt>>> {
    if call.call_type == CallType::Delegate {
        return None;
    }

    cheatnet_state
        .mocked_functions
        .get_mut(&call.storage_address)
        .and_then(|contract_functions| contract_functions.get_mut(&call.entry_point_selector))
}

fn mocked_call_info(call: CallEntryPoint, ret_data: Vec<Felt>) -> CallInfo {
    CallInfo {
        call: CallEntryPoint {
            class_hash: Some(call.class_hash.unwrap_or_default()),
            ..call
        },
        execution: CallExecution {
            retdata: Retdata(ret_data),
            events: vec![],
            l2_to_l1_messages: vec![],
            failed: false,
            gas_consumed: 0,
        },
        resources: ExecutionResources::default(),
        inner_calls: vec![],
        storage_read_values: vec![],
        accessed_storage_keys: HashSet::new(),
        read_class_hash_values: vec![],
        // This defaults to `CairoSteps` as of writing this, but it could be changed in the future
        tracked_resource: Default::default(),
        accessed_contract_addresses: Default::default(),
    }
}

fn aggregate_nested_syscall_counters(trace: &Rc<RefCell<CallTrace>>) -> SyscallCounter {
    let mut result = SyscallCounter::new();
    for nested_call_node in &trace.borrow().nested_calls {
        if let CallTraceNode::EntryPointCall(nested_call) = nested_call_node {
            let sub_trace_counter = aggregate_syscall_counters(nested_call);
            result = sum_syscall_counters(result, &sub_trace_counter);
        }
    }
    result
}

fn aggregate_syscall_counters(trace: &Rc<RefCell<CallTrace>>) -> SyscallCounter {
    let mut result = trace.borrow().used_syscalls.clone();
    for nested_call_node in &trace.borrow().nested_calls {
        if let CallTraceNode::EntryPointCall(nested_call) = nested_call_node {
            let sub_trace_counter = aggregate_nested_syscall_counters(nested_call);
            result = sum_syscall_counters(result, &sub_trace_counter);
        }
    }
    result
}

#[derive(Debug, Error)]
#[error("{}", source)]
pub struct EntryPointExecutionErrorWithTrace {
    pub source: EntryPointExecutionError,
    pub trace: Option<Vec<RelocatedTraceEntry>>,
}

impl<T> From<T> for EntryPointExecutionErrorWithTrace
where
    T: Into<EntryPointExecutionError>,
{
    fn from(value: T) -> Self {
        Self {
            source: value.into(),
            trace: None,
        }
    }
}

pub(crate) trait OnErrorLastPc<T>: Sized {
    fn on_error_get_last_pc(
        self,
        runner: &mut CairoRunner,
    ) -> Result<T, EntryPointExecutionErrorWithTrace>;
}

impl<T> OnErrorLastPc<T> for Result<T, EntryPointExecutionError> {
    fn on_error_get_last_pc(
        self,
        runner: &mut CairoRunner,
    ) -> Result<T, EntryPointExecutionErrorWithTrace> {
        match self {
            Err(source) => {
                let trace = get_relocated_vm_trace(runner);

                Err(EntryPointExecutionErrorWithTrace { source, trace })
            }
            Ok(value) => Ok(value),
        }
    }
}

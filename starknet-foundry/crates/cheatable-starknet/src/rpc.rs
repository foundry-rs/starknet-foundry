use anyhow::Result;
use cairo_vm::{hint_processor::hint_processor_definition::{HintProcessor, HintReference}, vm::{vm_core::VirtualMachine, runners::cairo_runner::RunResources, errors::{hint_errors::HintError, vm_errors::VirtualMachineError}}, types::exec_scope::ExecutionScopes, serde::deserialize_program::ApTracking};
use std::{sync::Arc, collections::HashMap, any::Any};

use blockifier::{state::{cached_state::CachedState, state_api::State}, execution::{entry_point::{CallEntryPoint, CallType, ExecutionResources, EntryPointExecutionContext, EntryPointExecutionResult, CallInfo, FAULTY_CLASS_HASH}, errors::{EntryPointExecutionError, PreExecutionError}, contract_class::{ContractClassV1, ContractClass}, cairo1_execution::{VmExecutionContext, initialize_execution_context, prepare_call_arguments, finalize_execution, run_entry_point}, syscalls::hint_processor::SyscallHintProcessor, common_hints::HintExecutionResult}};
use cairo_felt_blockifier::Felt252;
use starknet_api::{core::{ContractAddress, PatriciaKey, EntryPointSelector, ClassHash}, hash::{StarkFelt, StarkHash}, patricia_key, transaction::{Calldata, TransactionVersion}, deprecated_contract_class::EntryPointType};
use cairo_lang_casm::{hints::{Hint, StarknetHint}, operand::ResOperand};
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
        ContractClass::V0(_) => todo!(), // TODO redirect to original cairo handling
        ContractClass::V1(contract_class) => execute_entry_point_call_cairo1(
            call,
            contract_class,
            state,
            resources,
            context,
        ),
    }
}



pub struct CheatableSyscallHandler<'a> {
    pub syscall_handler: SyscallHintProcessor<'a>,
    pub rolled_contracts: HashMap<Felt252, Felt252>,
}

impl HintProcessor for CheatableSyscallHandler<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
        run_resources: &mut RunResources,
    ) -> HintExecutionResult {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();

        if let Some(Hint::Starknet(StarknetHint::SystemCall { system })) = maybe_extended_hint {
            return self.execute_syscall(system, vm);
        }
        self.syscall_handler
            .execute_hint(vm, exec_scopes, hint_data, constants, run_resources)
    }

    /// Trait function to store hint in the hint processor by string.
    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.syscall_handler.compile_hint(hint_code, ap_tracking_data, reference_ids, references)
    }
}

impl CheatableSyscallHandler<'_> {
    fn execute_syscall(
        &mut self,
        system: &ResOperand,
        vm: &mut VirtualMachine,
    ) -> Result<(), HintError> {
        let (cell, offset) = extract_buffer(system);
        let mut system_ptr = get_ptr(vm, cell, &offset)?;
    
        let selector = felt_from_pointer(vm, &mut system_ptr)
            .unwrap()
            .to_bytes_be();
    
        let gas_counter = usize_from_pointer(vm, &mut system_ptr).unwrap();
        let contract_address = felt_from_pointer(vm, &mut system_ptr).unwrap();
        let entry_point_selector = felt_from_pointer(vm, &mut system_ptr).unwrap();
    
        let start = relocatable_from_pointer(vm, &mut system_ptr).unwrap();
        let end = relocatable_from_pointer(vm, &mut system_ptr).unwrap();
        let calldata = read_data_from_range(vm, start, end).unwrap();
    
        let calldata_blockifier: Vec<blockifier_Felt252> = calldata.into_iter().map(|v| convert_to_blockifier_felt(v)).collect();
        assert_eq!(std::str::from_utf8(&selector).unwrap(), "CallContract");
        let result_blockfier: Vec<blockifier_Felt252> = call_contract(
            &convert_to_blockifier_felt(contract_address),
            &convert_to_blockifier_felt(entry_point_selector),
            &calldata_blockifier,
            blockifier_state,
        )
        .unwrap();
        let result: Vec<Felt252> = result_blockfier.into_iter().map(|v| convert_from_blockifier_felt(v)).collect();
    
        insert_at_pointer(vm, &mut system_ptr, gas_counter).unwrap();
        insert_at_pointer(vm, &mut system_ptr, Felt252::from(0)).unwrap();
    
        let mut ptr = vm.add_memory_segment();
        let start = ptr;
        for value in result {
            insert_at_pointer(vm, &mut ptr, value).unwrap();
        }
        let end = ptr;
    
        insert_at_pointer(vm, &mut system_ptr, start).unwrap();
        insert_at_pointer(vm, &mut system_ptr, end).unwrap();
    
        Ok(())
    }
}

fn felt_from_pointer(vm: &mut VirtualMachine, ptr: &mut Relocatable) -> Result<Felt252> {
    let entry_point_selector = vm.get_integer(*ptr)?.into_owned();
    *ptr += 1;
    Ok(entry_point_selector)
}

fn usize_from_pointer(vm: &mut VirtualMachine, ptr: &mut Relocatable) -> Result<usize> {
    let gas_counter = vm
        .get_integer(*ptr)?
        .to_usize()
        .ok_or_else(|| anyhow!("Failed to convert to usize"))?;
    *ptr += 1;
    Ok(gas_counter)
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
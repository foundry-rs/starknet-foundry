use anyhow::Result;
use cairo_vm::{hint_processor::hint_processor_definition::{HintProcessor, HintReference}, vm::{vm_core::VirtualMachine, runners::cairo_runner::RunResources, errors::{hint_errors::HintError, vm_errors::VirtualMachineError}}, types::{exec_scope::ExecutionScopes, relocatable::Relocatable}, serde::deserialize_program::ApTracking};
use std::{sync::Arc, collections::HashMap, any::Any};

use blockifier::{state::{cached_state::CachedState, state_api::State}, execution::{entry_point::{CallEntryPoint, CallType, ExecutionResources, EntryPointExecutionContext, EntryPointExecutionResult, CallInfo, FAULTY_CLASS_HASH}, errors::{EntryPointExecutionError, PreExecutionError}, contract_class::{ContractClassV1, ContractClass}, cairo1_execution::{VmExecutionContext, initialize_execution_context, prepare_call_arguments, finalize_execution, run_entry_point}, syscalls::hint_processor::{SyscallHintProcessor, SyscallExecutionError}, common_hints::HintExecutionResult, deprecated_syscalls::DeprecatedSyscallSelector, execution_utils::stark_felt_from_ptr}};
use cairo_felt_blockifier::Felt252;
use starknet_api::{core::{ContractAddress, PatriciaKey, EntryPointSelector, ClassHash}, hash::{StarkFelt, StarkHash}, patricia_key, transaction::{Calldata, TransactionVersion}, deprecated_contract_class::EntryPointType};
use cairo_lang_casm::{hints::{Hint, StarknetHint}, operand::{ResOperand, BinOpOperand, Operation, DerefOrImmediate, Register}};
use crate::{state::DictStateReader, constants::{TEST_ACCOUNT_CONTRACT_ADDRESS, build_transaction_context, build_block_context}};

use blockifier::execution::syscalls::{
    deploy, emit_event, get_block_hash, get_execution_info, keccak, library_call,
    library_call_l1_handler, replace_class, send_message_to_l1, storage_read, storage_write,
    StorageReadResponse, StorageWriteResponse, SyscallRequest, SyscallRequestWrapper,
    SyscallResponse, SyscallResponseWrapper, SyscallResult, 
};
type SyscallSelector = DeprecatedSyscallSelector;


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
            if let Some(Hint::Starknet(hint)) = maybe_extended_hint {
                return self.execute_next_syscall(vm, hint);
            }
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

/// Retrieves a [Relocatable] from the VM given a [ResOperand].
/// A [ResOperand] represents a CASM result expression, and is deserialized with the hint.
fn get_ptr_from_res_operand_unchecked(vm: &mut VirtualMachine, res: &ResOperand) -> Relocatable {
    let (cell, base_offset) = match res {
        ResOperand::Deref(cell) => (cell, Felt252::from(0)),
        ResOperand::BinOp(BinOpOperand {
            op: Operation::Add,
            a,
            b: DerefOrImmediate::Immediate(b),
        }) => (a, Felt252::from(b.clone().value)),
        _ => panic!("Illegal argument for a buffer."),
    };
    let base = match cell.register {
        Register::AP => vm.get_ap(),
        Register::FP => vm.get_fp(),
    };
    let cell_reloc = (base + (cell.offset as i32)).unwrap();
    (vm.get_relocatable(cell_reloc).unwrap() + &base_offset).unwrap()
}

impl CheatableSyscallHandler<'_> {
    fn peak_into

    pub fn execute_next_syscall(
        &mut self,
        vm: &mut VirtualMachine,
        hint: &StarknetHint,
    ) -> HintExecutionResult {
        let StarknetHint::SystemCall{ system: syscall } = hint else {
            return Err(HintError::CustomHint(
                "Test functions are unsupported on starknet.".into()
            ));
        };
        let initial_syscall_ptr = get_ptr_from_res_operand_unchecked(vm, syscall);
        self.verify_syscall_ptr(initial_syscall_ptr)?;

        let selector = SyscallSelector::try_from(self.read_next_syscall_selector(vm)?)?;
        

        // Keccak resource usage depends on the input length, so we increment the syscall count
        // in the syscall execution callback.
        if selector != SyscallSelector::Keccak {
            self.increment_syscall_count(&selector);
        }

        match selector {
            SyscallSelector::CallContract => {
                self.execute_syscall(vm, call_contract, constants::CALL_CONTRACT_GAS_COST)
            }
            SyscallSelector::Deploy => self.execute_syscall(vm, deploy, constants::DEPLOY_GAS_COST),
            SyscallSelector::EmitEvent => {
                self.execute_syscall(vm, emit_event, constants::EMIT_EVENT_GAS_COST)
            }
            SyscallSelector::GetBlockHash => {
                self.execute_syscall(vm, get_block_hash, constants::GET_BLOCK_HASH_GAS_COST)
            }
            SyscallSelector::GetExecutionInfo => {
                self.execute_syscall(vm, get_execution_info, constants::GET_EXECUTION_INFO_GAS_COST)
            }
            SyscallSelector::Keccak => self.execute_syscall(vm, keccak, constants::KECCAK_GAS_COST),
            SyscallSelector::LibraryCall => {
                self.execute_syscall(vm, library_call, constants::LIBRARY_CALL_GAS_COST)
            }
            SyscallSelector::LibraryCallL1Handler => {
                self.execute_syscall(vm, library_call_l1_handler, constants::LIBRARY_CALL_GAS_COST)
            }
            SyscallSelector::ReplaceClass => {
                self.execute_syscall(vm, replace_class, constants::REPLACE_CLASS_GAS_COST)
            }
            SyscallSelector::Secp256k1Add => {
                self.execute_syscall(vm, secp256k1_add, constants::SECP256K1_ADD_GAS_COST)
            }
            SyscallSelector::Secp256k1GetPointFromX => self.execute_syscall(
                vm,
                secp256k1_get_point_from_x,
                constants::SECP256K1_GET_POINT_FROM_X_GAS_COST,
            ),
            SyscallSelector::Secp256k1GetXy => {
                self.execute_syscall(vm, secp256k1_get_xy, constants::SECP256K1_GET_XY_GAS_COST)
            }
            SyscallSelector::Secp256k1Mul => {
                self.execute_syscall(vm, secp256k1_mul, constants::SECP256K1_MUL_GAS_COST)
            }
            SyscallSelector::Secp256k1New => {
                self.execute_syscall(vm, secp256k1_new, constants::SECP256K1_NEW_GAS_COST)
            }
            SyscallSelector::SendMessageToL1 => {
                self.execute_syscall(vm, send_message_to_l1, constants::SEND_MESSAGE_TO_L1_GAS_COST)
            }
            SyscallSelector::StorageRead => {
                self.execute_syscall(vm, storage_read, constants::STORAGE_READ_GAS_COST)
            }
            SyscallSelector::StorageWrite => {
                self.execute_syscall(vm, storage_write, constants::STORAGE_WRITE_GAS_COST)
            }
            _ => Err(HintError::UnknownHint(
                format!("Unsupported syscall selector {selector:?}.").into(),
            )),
        }
    }

    fn read_next_syscall_selector(&mut self, vm: &mut VirtualMachine) -> SyscallResult<StarkFelt> {
        let selector = stark_felt_from_ptr(vm, &mut self.syscall_ptr)?;

        Ok(selector)
    }

    fn verify_syscall_ptr(&self, actual_ptr: Relocatable) -> SyscallResult<()> {
        if actual_ptr != self.syscall_handler.syscall_ptr {
            return Err(SyscallExecutionError::BadSyscallPointer {
                expected_ptr: self.syscall_handler.syscall_ptr,
                actual_ptr,
            });
        }

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
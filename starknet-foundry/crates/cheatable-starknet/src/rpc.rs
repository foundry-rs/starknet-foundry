use anyhow::Result;
use cairo_vm::{hint_processor::hint_processor_definition::{HintProcessor, HintReference}, vm::{vm_core::VirtualMachine, runners::cairo_runner::{RunResources, CairoRunner, CairoArg}, errors::{hint_errors::HintError, vm_errors::VirtualMachineError}}, types::{exec_scope::ExecutionScopes, relocatable::{Relocatable, MaybeRelocatable}}, serde::deserialize_program::ApTracking};
use std::{sync::Arc, collections::HashMap, any::Any};

use blockifier::{state::{cached_state::CachedState, state_api::State}, execution::{entry_point::{CallEntryPoint, CallType, ExecutionResources, EntryPointExecutionContext, EntryPointExecutionResult, CallInfo, FAULTY_CLASS_HASH}, errors::{EntryPointExecutionError, PreExecutionError, VirtualMachineExecutionError}, contract_class::{ContractClassV1, ContractClass, EntryPointV1}, cairo1_execution::{VmExecutionContext, initialize_execution_context, prepare_call_arguments, finalize_execution, run_entry_point}, syscalls::{hint_processor::{SyscallHintProcessor, SyscallExecutionError, write_segment}, GetExecutionInfoResponse, EmptyRequest}, common_hints::HintExecutionResult, deprecated_syscalls::{DeprecatedSyscallSelector}, execution_utils::{stark_felt_from_ptr, Args}}};
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

pub enum CallContractOutput {
    Success { ret_data: Vec<Felt252> },
    Panic { panic_data: Vec<Felt252> },
}

// This can mutate state, the name of the syscall is not very good
pub fn call_contract(
    contract_address: &Felt252,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
    blockifier_state: &mut CachedState<DictStateReader>,
) -> Result<CallContractOutput> {
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

    // let exec_result = entry_point.execute(blockifier_state, &mut resources, &mut context);

    let exec_result = execute_call_entry_point(entry_point, blockifier_state, &mut resources, &mut context);
       

    if let Ok(call_info) = exec_result {
        let raw_return_data = &call_info.execution.retdata.0;

        let return_data = raw_return_data
            .iter()
            .map(|data| Felt252::from_bytes_be(data.bytes()))
            .collect();

        Ok(CallContractOutput::Success {
            ret_data: return_data,
        })
    } else if let Err(EntryPointExecutionError::ExecutionFailed { error_data }) = exec_result {
        let err_data = error_data
            .iter()
            .map(|data| Felt252::from_bytes_be(data.bytes()))
            .collect();

        Ok(CallContractOutput::Panic {
            panic_data: err_data,
        })
    } else {
        panic!("Unparseable result: {exec_result:?}");
    }
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
                return self.execute_next_syscall_xd(vm, hint);
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

    pub fn is_cheated(&mut self, vm: &mut VirtualMachine, selector: SyscallSelector) -> bool {
        match selector {
           SyscallSelector::GetExecutionInfo => true,
           _ => false 
        }
    }

    pub fn execute_next_syscall_xd(
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

        if selector != SyscallSelector::Keccak {
            self.syscall_handler.increment_syscall_count_by(&selector, 1);
        }

        if self.is_cheated(vm, selector) {
            let execution_info_ptr = self.syscall_handler.get_or_allocate_execution_info_segment(vm)?;
            let data = vm.get_range(execution_info_ptr, 1)[0].clone();
            if let MaybeRelocatable::RelocatableValue(block_info_ptr)  = data.unwrap().into_owned() {
                let ptr_cheated_exec_info = vm.add_memory_segment();
                let ptr_cheated_block_info = vm.add_memory_segment();

                // create a new segment with replaced block info
                let ptr_cheated_block_info_c = vm.load_data(ptr_cheated_block_info, &vec![MaybeRelocatable::Int(Felt252::from(123))]).unwrap(); 
                let orginal_block_info = vm.get_continuous_range((block_info_ptr + 1 as usize).unwrap() as Relocatable, 2).unwrap();
                vm.load_data(ptr_cheated_block_info_c, &orginal_block_info).unwrap();


                // create a new segment with replaced execution_info including pointer to updated block info
                let ptr_cheated_exec_info_c = vm.load_data(ptr_cheated_exec_info, &vec![MaybeRelocatable::RelocatableValue(ptr_cheated_block_info)]).unwrap(); 
                let original_execution_info = vm.get_continuous_range((execution_info_ptr + 1 as usize).unwrap() as Relocatable, 4).unwrap();
                vm.load_data(ptr_cheated_exec_info_c, &original_execution_info).unwrap();


                let SyscallRequestWrapper { gas_counter, request: _ } =
                    SyscallRequestWrapper::<EmptyRequest>::read(vm, &mut self.syscall_handler.syscall_ptr)?;

                let response = GetExecutionInfoResponse { execution_info_ptr: ptr_cheated_exec_info };
                let response_w = SyscallResponseWrapper::Success { gas_counter, response };
                response_w.write(vm, &mut self.syscall_handler.syscall_ptr)?;
                return Ok(());
            }
        }
        self.syscall_handler.execute_next_syscall(vm, hint)
    }

    fn read_next_syscall_selector(&mut self, vm: &mut VirtualMachine) -> SyscallResult<StarkFelt> {
        let selector = stark_felt_from_ptr(vm, &mut self.syscall_handler.syscall_ptr)?;
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

    let mut syscall_hh = CheatableSyscallHandler {
        syscall_handler,
        rolled_contracts: HashMap::new()
    };

    // Execute.
    run_entry_point_m(
        &mut vm,
        &mut runner,
        &mut syscall_hh,
        entry_point,
        args,
        program_segment_size,
    )?;
    let sys_h = syscall_hh.syscall_handler; 
    let call_info =
        finalize_execution(vm, runner, sys_h, previous_vm_resources, n_total_args)?;
    if call_info.execution.failed {
        return Err(EntryPointExecutionError::ExecutionFailed {
            error_data: call_info.execution.retdata.0,
        });
    }

    Ok(call_info)
}

/// Runs the runner from the given PC.
pub fn run_entry_point_m(
    vm: &mut VirtualMachine,
    runner: &mut CairoRunner,
    hint_processor: &mut dyn HintProcessor,
    entry_point: EntryPointV1,
    args: Args,
    program_segment_size: usize,
) -> Result<(), VirtualMachineExecutionError> {
    // let mut run_resources = hint_processor.context.vm_run_resources.clone(); // TODO
    let verify_secure = true;
    let args: Vec<&CairoArg> = args.iter().collect();

    let mut run_resources = RunResources::default();
    let result = runner.run_from_entrypoint(
        entry_point.pc(),
        &args,
        &mut run_resources,
        verify_secure,
        Some(program_segment_size),
        vm,
        hint_processor,
    );

    // hint_processor.context.vm_run_resources = run_resources; // TODO 
    Ok(result?)
}
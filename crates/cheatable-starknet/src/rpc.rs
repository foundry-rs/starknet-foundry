use anyhow::Result;
use cairo_vm::{
    hint_processor::hint_processor_definition::{HintProcessor, HintReference},
    serde::deserialize_program::ApTracking,
    types::{
        exec_scope::ExecutionScopes,
        relocatable::{MaybeRelocatable, Relocatable},
    },
    vm::{
        errors::{hint_errors::HintError, vm_errors::VirtualMachineError},
        runners::cairo_runner::{CairoArg, CairoRunner, RunResources},
        vm_core::VirtualMachine,
    },
};
use std::{any::Any, collections::HashMap, sync::Arc};

use crate::{
    constants::{build_block_context, build_transaction_context, TEST_ACCOUNT_CONTRACT_ADDRESS},
    state::DictStateReader,
};
use blockifier::{
    abi::constants,
    execution::{
        execution_utils::ReadOnlySegment,
        syscalls::{
            hint_processor::{create_retdata_segment, write_segment, OUT_OF_GAS_ERROR},
            CallContractRequest, WriteResponseResult,
        },
    },
    transaction::transaction_utils::update_remaining_gas,
};
use blockifier::{
    execution::{
        cairo1_execution::{
            finalize_execution, initialize_execution_context, prepare_call_arguments,
            VmExecutionContext,
        },
        common_hints::HintExecutionResult,
        contract_class::{ContractClass, ContractClassV1, EntryPointV1},
        deprecated_syscalls::DeprecatedSyscallSelector,
        entry_point::{
            CallEntryPoint, CallInfo, CallType, EntryPointExecutionContext,
            EntryPointExecutionResult, ExecutionResources, FAULTY_CLASS_HASH,
        },
        errors::{EntryPointExecutionError, PreExecutionError, VirtualMachineExecutionError},
        execution_utils::{felt_to_stark_felt, stark_felt_to_felt, Args},
        syscalls::{
            hint_processor::{SyscallExecutionError, SyscallHintProcessor},
            EmptyRequest, GetExecutionInfoResponse,
        },
    },
    state::{cached_state::CachedState, state_api::State},
};
use cairo_felt::Felt252;
use cairo_lang_casm::{
    hints::{Hint, StarknetHint},
    operand::{BinOpOperand, DerefOrImmediate, Operation, Register, ResOperand},
};
use starknet_api::{
    core::{ClassHash, ContractAddress, EntryPointSelector, PatriciaKey},
    deprecated_contract_class::EntryPointType,
    hash::{StarkFelt, StarkHash},
    patricia_key,
    transaction::{Calldata, TransactionVersion},
};

use crate::cheatcodes::deploy::felt_from_short_string;
use blockifier::execution::syscalls::{
    LibraryCallRequest, SyscallRequest, SyscallRequestWrapper, SyscallResponse,
    SyscallResponseWrapper, SyscallResult,
};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_starknet::contract::starknet_keccak;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessorLogic;
use cairo_vm::vm::runners::cairo_runner::ResourceTracker;
use num_bigint::BigUint;

use crate::CheatedState;

type SyscallSelector = DeprecatedSyscallSelector;

pub enum CallContractOutput {
    Success { ret_data: Vec<Felt252> },
    Panic { panic_data: Vec<Felt252> },
}

// This does contract call without the transaction layer. This way `call_contract` can return data and modify state.
// `call` and `invoke` on the transactional layer use such method under the hood.
pub fn call_contract(
    contract_address: &Felt252,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
    blockifier_state: &mut CachedState<DictStateReader>,
    cheated_state: &mut CheatedState,
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
    let mut entry_point = CallEntryPoint {
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
        block_context.invoke_tx_max_n_steps.try_into().unwrap(),
    );

    let exec_result = execute_call_entry_point(
        &mut entry_point,
        blockifier_state,
        cheated_state,
        &mut resources,
        &mut context,
    );

    if let Ok(call_info) = exec_result {
        let panic_data = check_emitted_events(cheated_state, &call_info);
        if !panic_data.is_empty() {
            return Ok(CallContractOutput::Panic { panic_data });
        }

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

// Copied over (with modifications) from blockifier/src/execution/entry_point.rs:144
fn execute_call_entry_point(
    entry_point: &mut CallEntryPoint,
    state: &mut dyn State,
    cheated_state: &CheatedState,
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> EntryPointExecutionResult<CallInfo> {
    // We skip recursion depth validation here.

    // Validate contract is deployed.
    let storage_address = entry_point.storage_address;
    let storage_class_hash = state.get_class_hash_at(entry_point.storage_address)?;
    if storage_class_hash == ClassHash::default() {
        return Err(
            PreExecutionError::UninitializedStorageAddress(entry_point.storage_address).into(),
        );
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
    entry_point.class_hash = Some(class_hash);
    let contract_class = state.get_compiled_contract_class(&class_hash)?;

    let result = match contract_class {
        ContractClass::V0(_) => panic!("Cairo 0 classes are not supported"),
        ContractClass::V1(contract_class) => execute_entry_point_call_cairo1(
            entry_point.clone(),
            &contract_class,
            state,
            cheated_state,
            resources,
            context,
        ),
    };

    result.map_err(|error| {
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
    })
}

pub struct CheatableSyscallHandler<'a> {
    pub syscall_handler: SyscallHintProcessor<'a>,
    pub cheated_state: &'a CheatedState,
}

impl ResourceTracker for CheatableSyscallHandler<'_> {
    fn consumed(&self) -> bool {
        self.syscall_handler.context.vm_run_resources.consumed()
    }

    fn consume_step(&mut self) {
        self.syscall_handler.context.vm_run_resources.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.syscall_handler.context.vm_run_resources.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.syscall_handler
            .context
            .vm_run_resources
            .run_resources()
    }
}

impl HintProcessorLogic for CheatableSyscallHandler<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> HintExecutionResult {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();

        if let Some(Hint::Starknet(StarknetHint::SystemCall { .. })) = maybe_extended_hint {
            if let Some(Hint::Starknet(hint)) = maybe_extended_hint {
                return self.execute_next_syscall_cheated(vm, hint);
            }
        }
        self.syscall_handler
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }

    /// Trait function to store hint in the hint processor by string.
    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.syscall_handler
            .compile_hint(hint_code, ap_tracking_data, reference_ids, references)
    }
}

/// Retrieves a [Relocatable] from the VM given a [`ResOperand`].
/// A [`ResOperand`] represents a CASM result expression, and is deserialized with the hint.
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
    let cell_reloc = (base + i32::from(cell.offset)).unwrap();
    (vm.get_relocatable(cell_reloc).unwrap() + &base_offset).unwrap()
}

fn stark_felt_from_ptr_immutable(
    vm: &VirtualMachine,
    ptr: &Relocatable,
) -> Result<StarkFelt, VirtualMachineError> {
    Ok(felt_to_stark_felt(&felt_from_ptr_immutable(vm, ptr)?))
}

fn felt_from_ptr_immutable(
    vm: &VirtualMachine,
    ptr: &Relocatable,
) -> Result<Felt252, VirtualMachineError> {
    let felt = vm.get_integer(*ptr)?.into_owned();
    Ok(felt)
}

impl CheatableSyscallHandler<'_> {
    fn get_cheated_block_info_ptr(
        &self,
        vm: &mut VirtualMachine,
        original_block_info: &[MaybeRelocatable],
        contract_address: &ContractAddress,
    ) -> Relocatable {
        // create a new segment with replaced block info
        let ptr_cheated_block_info = vm.add_memory_segment();

        let mut new_block_info = original_block_info.to_owned();

        if let Some(rolled_number) = self.cheated_state.rolled_contracts.get(contract_address) {
            new_block_info[0] = MaybeRelocatable::Int(rolled_number.clone());
        };

        if let Some(warped_timestamp) = self.cheated_state.warped_contracts.get(contract_address) {
            new_block_info[1] = MaybeRelocatable::Int(warped_timestamp.clone());
        };

        vm.load_data(ptr_cheated_block_info, &new_block_info)
            .unwrap();
        ptr_cheated_block_info
    }

    fn address_is_pranked(
        &mut self,
        _vm: &mut VirtualMachine,
        contract_address: &ContractAddress,
    ) -> bool {
        self.cheated_state
            .pranked_contracts
            .contains_key(contract_address)
    }

    fn address_is_warped(
        &mut self,
        _vm: &mut VirtualMachine,
        contract_address: &ContractAddress,
    ) -> bool {
        self.cheated_state
            .warped_contracts
            .contains_key(contract_address)
    }

    fn address_is_rolled(
        &mut self,
        _vm: &mut VirtualMachine,
        contract_address: &ContractAddress,
    ) -> bool {
        self.cheated_state
            .rolled_contracts
            .contains_key(contract_address)
    }

    fn address_is_cheated(
        &mut self,
        vm: &mut VirtualMachine,
        selector: SyscallSelector,
        contract_address: &ContractAddress,
    ) -> bool {
        match selector {
            SyscallSelector::GetExecutionInfo => {
                self.address_is_rolled(vm, contract_address)
                    || self.address_is_pranked(vm, contract_address)
                    || self.address_is_warped(vm, contract_address)
            }
            _ => false,
        }
    }

    fn execute_next_syscall_cheated(
        &mut self,
        vm: &mut VirtualMachine,
        hint: &StarknetHint,
    ) -> HintExecutionResult {
        // We peak into the selector without incrementing the pointer as it is done later
        let syscall_selector_pointer = self.syscall_handler.syscall_ptr;
        let selector = SyscallSelector::try_from(stark_felt_from_ptr_immutable(
            vm,
            &syscall_selector_pointer,
        )?)?;
        let contract_address = self.syscall_handler.storage_address();

        if self.address_is_cheated(vm, selector, &contract_address) {
            let StarknetHint::SystemCall { system: syscall } = hint else {
                return Err(HintError::CustomHint(
                    "Test functions are unsupported on starknet.".into(),
                ));
            };
            let initial_syscall_ptr = get_ptr_from_res_operand_unchecked(vm, syscall);
            self.verify_syscall_ptr(initial_syscall_ptr)?;

            // Increment, since the selector was peeked into before
            self.syscall_handler.syscall_ptr += 1;

            if selector != SyscallSelector::Keccak {
                self.syscall_handler
                    .increment_syscall_count_by(&selector, 1);
            }

            let SyscallRequestWrapper {
                gas_counter,
                request: _,
            } = SyscallRequestWrapper::<EmptyRequest>::read(
                vm,
                &mut self.syscall_handler.syscall_ptr,
            )?;

            // ExecutionInfo from corelib/src/starknet/info.cairo
            // block_info, tx_info, caller_address, contract_address, entry_point_selector

            let execution_info_ptr = self
                .syscall_handler
                .get_or_allocate_execution_info_segment(vm)?;

            let ptr_cheated_exec_info = vm.add_memory_segment();

            // Initialize as old exec_info
            let mut new_exec_info = vm.get_continuous_range(execution_info_ptr, 5).unwrap();

            if self.address_is_rolled(vm, &contract_address)
                || self.address_is_warped(vm, &contract_address)
            {
                let data = vm.get_range(execution_info_ptr, 1)[0].clone();
                if let MaybeRelocatable::RelocatableValue(block_info_ptr) =
                    data.unwrap().into_owned()
                {
                    let original_block_info = vm.get_continuous_range(block_info_ptr, 3).unwrap();

                    let ptr_cheated_block_info = self.get_cheated_block_info_ptr(
                        vm,
                        &original_block_info,
                        &contract_address,
                    );

                    new_exec_info[0] = MaybeRelocatable::RelocatableValue(ptr_cheated_block_info);
                }
            }

            if self.address_is_pranked(vm, &contract_address) {
                new_exec_info[2] = MaybeRelocatable::Int(stark_felt_to_felt(
                    *self
                        .cheated_state
                        .pranked_contracts
                        .get(&contract_address)
                        .expect("No caller address value found for the pranked contract address")
                        .0
                        .key(),
                ));
            }

            vm.load_data(ptr_cheated_exec_info, &new_exec_info).unwrap();

            let remaining_gas = gas_counter - constants::GET_EXECUTION_INFO_GAS_COST;
            let response = GetExecutionInfoResponse {
                execution_info_ptr: ptr_cheated_exec_info,
            };
            let response_wrapper = SyscallResponseWrapper::Success {
                gas_counter: remaining_gas,
                response,
            };
            response_wrapper.write(vm, &mut self.syscall_handler.syscall_ptr)?;
            return Ok(());
        } else if SyscallSelector::CallContract == selector {
            // Increment, since the selector was peeked into before
            self.syscall_handler.syscall_ptr += 1;
            return self.execute_syscall(
                vm,
                call_contract_syscall,
                constants::CALL_CONTRACT_GAS_COST,
            );
        } else if SyscallSelector::LibraryCall == selector {
            // Increment, since the selector was peeked into before
            self.syscall_handler.syscall_ptr += 1;
            return self.execute_syscall(
                vm,
                library_call_syscall,
                constants::CALL_CONTRACT_GAS_COST,
            );
        }

        self.syscall_handler.execute_next_syscall(vm, hint)
    }

    // Exactly the same implementation as SyscallHintProcessor::execute_syscall
    // It is copied because it is private in SyscallHintProcessor
    // and we need to call it here to override syscalls logic.
    fn execute_syscall<Request, Response, ExecuteCallback>(
        &mut self,
        vm: &mut VirtualMachine,
        execute_callback: ExecuteCallback,
        base_gas_cost: u64,
    ) -> HintExecutionResult
    where
        Request: SyscallRequest + std::fmt::Debug,
        Response: SyscallResponse + std::fmt::Debug,
        ExecuteCallback: FnOnce(
            Request,
            &mut VirtualMachine,
            &mut CheatableSyscallHandler<'_>,
            &mut u64, // Remaining gas.
        ) -> SyscallResult<Response>,
    {
        let SyscallRequestWrapper {
            gas_counter,
            request,
        } = SyscallRequestWrapper::<Request>::read(vm, &mut self.syscall_handler.syscall_ptr)?;

        if gas_counter < base_gas_cost {
            //  Out of gas failure.
            let out_of_gas_error =
                StarkFelt::try_from(OUT_OF_GAS_ERROR).map_err(SyscallExecutionError::from)?;
            let response: SyscallResponseWrapper<Response> = SyscallResponseWrapper::Failure {
                gas_counter,
                error_data: vec![out_of_gas_error],
            };
            response.write(vm, &mut self.syscall_handler.syscall_ptr)?;

            return Ok(());
        }

        // Execute.
        let mut remaining_gas = gas_counter - base_gas_cost;
        let original_response = execute_callback(request, vm, self, &mut remaining_gas);
        let response = match original_response {
            Ok(response) => SyscallResponseWrapper::Success {
                gas_counter: remaining_gas,
                response,
            },
            Err(SyscallExecutionError::SyscallError { error_data: data }) => {
                SyscallResponseWrapper::Failure {
                    gas_counter: remaining_gas,
                    error_data: data,
                }
            }
            Err(error) => return Err(error.into()),
        };

        response.write(vm, &mut self.syscall_handler.syscall_ptr)?;

        Ok(())
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

#[derive(Debug)]
// Inspired by blockifier::execution::syscalls::SingleSegmentResponse
// It is created here because fields in the original structure are private
// so we cannot create it in call_contract_syscall
pub struct SingleSegmentResponse {
    segment: ReadOnlySegment,
}

impl SyscallResponse for SingleSegmentResponse {
    fn write(self, vm: &mut VirtualMachine, ptr: &mut Relocatable) -> WriteResponseResult {
        write_segment(vm, ptr, self.segment)
    }
}

// Inspired by blockifier::execution::syscalls::call_contract
// Calls a contract using our implementation with modified logic
pub fn call_contract_syscall(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut CheatableSyscallHandler<'_>,
    remaining_gas: &mut u64,
) -> SyscallResult<SingleSegmentResponse> {
    let storage_address = request.contract_address;
    let mut entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(storage_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector: request.function_selector,
        calldata: request.calldata,
        storage_address,
        caller_address: syscall_handler.syscall_handler.storage_address(),
        call_type: CallType::Call,
        initial_gas: *remaining_gas,
    };
    let retdata_segment = execute_inner_call(&mut entry_point, vm, syscall_handler, remaining_gas)?;

    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
}

// Inspired by blockifier::execution::syscalls::library_call
// Calls a library using our implementation with modified logic
pub fn library_call_syscall(
    request: LibraryCallRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut CheatableSyscallHandler<'_>,
    remaining_gas: &mut u64,
) -> SyscallResult<SingleSegmentResponse> {
    let mut entry_point = CallEntryPoint {
        class_hash: Some(request.class_hash),
        code_address: None,
        entry_point_type: EntryPointType::External,
        entry_point_selector: request.function_selector,
        calldata: request.calldata,
        // The call context remains the same in a library call.
        storage_address: syscall_handler.syscall_handler.storage_address(),
        caller_address: syscall_handler.syscall_handler.caller_address(),
        call_type: CallType::Delegate,
        initial_gas: *remaining_gas,
    };

    let retdata_segment = execute_inner_call(&mut entry_point, vm, syscall_handler, remaining_gas)?;

    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
}

// Inspired by blockifier::hint_processor::execute_inner_call
pub fn execute_inner_call(
    call: &mut CallEntryPoint,
    vm: &mut VirtualMachine,
    syscall_handler: &mut CheatableSyscallHandler<'_>,
    remaining_gas: &mut u64,
) -> SyscallResult<ReadOnlySegment> {
    // Modified code
    let call_info = execute_call_entry_point(
        call,
        syscall_handler.syscall_handler.state,
        syscall_handler.cheated_state,
        syscall_handler.syscall_handler.resources,
        syscall_handler.syscall_handler.context,
    )?;
    // Modified end

    let raw_retdata = &call_info.execution.retdata.0;

    if call_info.execution.failed {
        // TODO(spapini): Append an error word according to starknet spec if needed.
        // Something like "EXECUTION_ERROR".
        return Err(SyscallExecutionError::SyscallError {
            error_data: raw_retdata.clone(),
        });
    }

    let retdata_segment =
        create_retdata_segment(vm, &mut syscall_handler.syscall_handler, raw_retdata)?;
    update_remaining_gas(remaining_gas, &call_info);

    syscall_handler.syscall_handler.inner_calls.push(call_info);

    Ok(retdata_segment)
}

/// Executes a specific call to a contract entry point and returns its output.
fn execute_entry_point_call_cairo1(
    call: CallEntryPoint,
    contract_class: &ContractClassV1,
    state: &mut dyn State,
    cheated_state: &CheatedState,
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> EntryPointExecutionResult<CallInfo> {
    let VmExecutionContext {
        mut runner,
        mut vm,
        mut syscall_handler,
        initial_syscall_ptr,
        entry_point,
        program_extra_data_length,
    } = initialize_execution_context(call, contract_class, state, resources, context)?;

    let args = prepare_call_arguments(
        &syscall_handler.call,
        &mut vm,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
        &entry_point,
    )?;
    let n_total_args = args.len();

    // Snapshot the VM resources, in order to calculate the usage of this run at the end.
    let previous_vm_resources = syscall_handler.resources.vm_resources.clone();

    let mut cheatable_syscall_handler = CheatableSyscallHandler {
        syscall_handler,
        cheated_state,
    };

    // Execute.
    cheatable_run_entry_point(
        &mut vm,
        &mut runner,
        &mut cheatable_syscall_handler,
        &entry_point,
        &args,
        program_extra_data_length,
    )?;

    let call_info = finalize_execution(
        vm,
        runner,
        cheatable_syscall_handler.syscall_handler,
        previous_vm_resources,
        n_total_args,
        program_extra_data_length,
    )?;
    if call_info.execution.failed {
        return Err(EntryPointExecutionError::ExecutionFailed {
            error_data: call_info.execution.retdata.0,
        });
    }

    Ok(call_info)
}

/// Runs the runner from the given PC.
fn cheatable_run_entry_point(
    vm: &mut VirtualMachine,
    runner: &mut CairoRunner,
    hint_processor: &mut dyn HintProcessor,
    entry_point: &EntryPointV1,
    args: &Args,
    program_segment_size: usize,
) -> Result<(), VirtualMachineExecutionError> {
    let verify_secure = false;
    let args: Vec<&CairoArg> = args.iter().collect();

    runner.run_from_entrypoint(
        entry_point.pc(),
        &args,
        verify_secure,
        Some(program_segment_size),
        vm,
        hint_processor,
    )?;

    Ok(())
}

/// Check if events defined in `expect_events` cheatcode were emitted
fn check_emitted_events(cheated_state: &mut CheatedState, call_info: &CallInfo) -> Vec<Felt252> {
    for expected_event in &cheated_state.expected_events {
        let mut found = false;
        for event in &call_info.execution.events {
            if starknet_keccak(
                as_cairo_short_string(&expected_event.name)
                    .unwrap()
                    .as_bytes(),
            ) == BigUint::from_bytes_be(event.event.keys[0].0.bytes())
            {
                let keys: Vec<Felt252> = event
                    .event
                    .keys
                    .iter()
                    .map(|stark_felt| stark_felt_to_felt(stark_felt.0))
                    .collect();
                if expected_event.keys != keys[1..] {
                    return vec![felt_from_short_string("Expected keys differ from real")];
                }
                let data: Vec<Felt252> = event
                    .event
                    .data
                    .0
                    .iter()
                    .map(|stark_felt| stark_felt_to_felt(*stark_felt))
                    .collect();
                if expected_event.data != data {
                    return vec![felt_from_short_string("Expected data differs from real")];
                }

                found = true;
                break;
            }
        }
        if !found {
            return vec![
                felt_from_short_string("Expected event was not emitted"),
                expected_event.name.clone(),
            ];
        }
    }
    cheated_state.expected_events = vec![];
    vec![]
}

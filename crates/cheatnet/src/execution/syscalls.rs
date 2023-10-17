use crate::cheatcodes::spy_events::Event;
use crate::execution::{
    contract_print::{contract_print, PrintingResult},
    entry_point::execute_constructor_entry_point,
};
use crate::state::CheatnetState;
use anyhow::Result;
use blockifier::execution::syscalls::{
    DeployRequest, DeployResponse, LibraryCallRequest, SyscallRequest, SyscallRequestWrapper,
    SyscallResponse, SyscallResponseWrapper, SyscallResult,
};
use blockifier::execution::{call_info::CallInfo, entry_point::ConstructorContext};
use blockifier::state::errors::StateError;
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
        common_hints::HintExecutionResult,
        deprecated_syscalls::DeprecatedSyscallSelector,
        entry_point::{
            CallEntryPoint, CallType, EntryPointExecutionContext, EntryPointExecutionResult,
            ExecutionResources,
        },
        execution_utils::felt_to_stark_felt,
        syscalls::{
            hint_processor::{SyscallExecutionError, SyscallHintProcessor},
            EmptyRequest, GetExecutionInfoResponse,
        },
    },
    state::state_api::State,
};
use cairo_felt::Felt252;
use cairo_lang_casm::{
    hints::{Hint, StarknetHint},
    operand::{BinOpOperand, DerefOrImmediate, Operation, Register, ResOperand},
};
use cairo_vm::hint_processor::hint_processor_definition::HintProcessorLogic;
use cairo_vm::vm::runners::cairo_runner::ResourceTracker;
use cairo_vm::{
    hint_processor::hint_processor_definition::HintReference,
    serde::deserialize_program::ApTracking,
    types::{exec_scope::ExecutionScopes, relocatable::Relocatable},
    vm::{
        errors::{hint_errors::HintError, vm_errors::VirtualMachineError},
        runners::cairo_runner::RunResources,
        vm_core::VirtualMachine,
    },
};
use starknet_api::core::calculate_contract_address;
use starknet_api::{
    core::{ClassHash, ContractAddress},
    deprecated_contract_class::EntryPointType,
    hash::StarkFelt,
    transaction::Calldata,
};
use std::{any::Any, collections::HashMap};

use super::calls::{execute_inner_call, execute_library_call};
use super::execution_info::get_cheated_exec_info_ptr;
pub type SyscallSelector = DeprecatedSyscallSelector;

#[derive(Debug)]
// crates/blockifier/src/execution/syscalls/mod.rs:127 (SingleSegmentResponse)
// It is created here because fields in the original structure are private
// so we cannot create it in call_contract_syscall
pub struct SingleSegmentResponse {
    pub(crate) segment: ReadOnlySegment,
}
// crates/blockifier/src/execution/syscalls/mod.rs:131 (SyscallResponse for SingleSegmentResponse)
impl SyscallResponse for SingleSegmentResponse {
    fn write(self, vm: &mut VirtualMachine, ptr: &mut Relocatable) -> WriteResponseResult {
        write_segment(vm, ptr, self.segment)
    }
}

// blockifier/src/execution/syscalls/mod.rs:157 (call_contract)
pub fn call_contract_syscall(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut CheatableSyscallHandler<'_>, // Modified parameter type
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

    // region: Modified blockifier code
    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
    // endregion
}

// This hint processor modifies the standard syscalls implementation to react upon changes
// introduced by cheatcodes that e.g. returns mocked data
// If it cannot execute a cheatcode it falls back to SyscallHintProcessor, which provides standard implementation of
// hints from blockifier
pub struct CheatableSyscallHandler<'a> {
    pub syscall_handler: SyscallHintProcessor<'a>,
    pub cheatnet_state: &'a mut CheatnetState,
}

impl<'a> CheatableSyscallHandler<'a> {
    pub fn new(
        syscall_handler: SyscallHintProcessor<'a>,
        cheatnet_state: &'a mut CheatnetState,
    ) -> Self {
        CheatableSyscallHandler {
            syscall_handler,
            cheatnet_state,
        }
    }
}

// crates/blockifier/src/execution/syscalls/hint_processor.rs:472 (ResourceTracker for SyscallHintProcessor)
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

        match contract_print(vm, maybe_extended_hint) {
            PrintingResult::Printed => return Ok(()),
            PrintingResult::Passed => (),
            PrintingResult::Err(err) => return Err(err),
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

// crates/blockifier/src/execution/syscalls/hint_processor.rs:454
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

pub fn stark_felt_from_ptr_immutable(
    vm: &VirtualMachine,
    ptr: &Relocatable,
) -> Result<StarkFelt, VirtualMachineError> {
    Ok(felt_to_stark_felt(&felt_from_ptr_immutable(vm, ptr)?))
}

pub fn felt_from_ptr_immutable(
    vm: &VirtualMachine,
    ptr: &Relocatable,
) -> Result<Felt252, VirtualMachineError> {
    let felt = vm.get_integer(*ptr)?.into_owned();
    Ok(felt)
}

fn get_syscall_operand(hint: &StarknetHint) -> Result<&ResOperand, HintError> {
    let StarknetHint::SystemCall { system: syscall } = hint else {
        return Err(HintError::CustomHint(
            "Test functions are unsupported on starknet.".into(),
        ));
    };
    Ok(syscall)
}

impl CheatableSyscallHandler<'_> {
    fn execute_next_syscall_cheated(
        &mut self,
        vm: &mut VirtualMachine,
        hint: &StarknetHint,
    ) -> HintExecutionResult {
        // We peak into the selector without incrementing the pointer as it is done later
        let syscall = get_syscall_operand(hint)?;
        let initial_syscall_ptr = get_ptr_from_res_operand_unchecked(vm, syscall);
        let selector =
            SyscallSelector::try_from(stark_felt_from_ptr_immutable(vm, &initial_syscall_ptr)?)?;
        self.verify_syscall_ptr(initial_syscall_ptr)?;
        let contract_address = self.syscall_handler.storage_address();

        if SyscallSelector::GetExecutionInfo == selector
            && self.cheatnet_state.address_is_cheated(&contract_address)
        {
            // Increment, since the selector was peeked into before
            self.syscall_handler.syscall_ptr += 1;
            self.syscall_handler
                .increment_syscall_count_by(&selector, 1);

            let SyscallRequestWrapper {
                gas_counter,
                request: _,
            } = SyscallRequestWrapper::<EmptyRequest>::read(
                vm,
                &mut self.syscall_handler.syscall_ptr,
            )?;

            let execution_info_ptr = self
                .syscall_handler
                .get_or_allocate_execution_info_segment(vm)?;

            let ptr_cheated_exec_info = get_cheated_exec_info_ptr(
                self.cheatnet_state,
                vm,
                execution_info_ptr,
                &contract_address,
            );

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
            self.syscall_handler
                .increment_syscall_count_by(&selector, 1);

            return self.execute_syscall(
                vm,
                call_contract_syscall,
                constants::CALL_CONTRACT_GAS_COST,
            );
        } else if SyscallSelector::LibraryCall == selector {
            // Increment, since the selector was peeked into before
            self.syscall_handler.syscall_ptr += 1;
            self.syscall_handler
                .increment_syscall_count_by(&selector, 1);

            return self.execute_syscall(
                vm,
                library_call_syscall,
                constants::CALL_CONTRACT_GAS_COST,
            );
        } else if SyscallSelector::Deploy == selector {
            // Increment, since the selector was peeked into before
            self.syscall_handler.syscall_ptr += 1;
            self.syscall_handler
                .increment_syscall_count_by(&selector, 1);

            return self.execute_syscall(vm, deploy_syscall, constants::DEPLOY_GAS_COST);
        } else if SyscallSelector::EmitEvent == selector {
            // No incrementation, since execute_next_syscall reads selector and increments syscall_ptr
            let events_len_before = self.syscall_handler.events.len();
            let result = self.syscall_handler.execute_next_syscall(vm, hint);

            if result.is_ok() {
                assert_eq!(
                    events_len_before + 1,
                    self.syscall_handler.events.len(),
                    "EmitEvent syscall is expected to emit exactly one event"
                );
                let contract_address = self
                    .syscall_handler
                    .call
                    .code_address
                    .unwrap_or(self.syscall_handler.call.storage_address);

                for spy_on in &mut self.cheatnet_state.spies {
                    if spy_on.does_spy(contract_address) {
                        let event = Event::from_ordered_event(
                            self.syscall_handler.events.last().unwrap(),
                            contract_address,
                        );
                        self.cheatnet_state.detected_events.push(event);
                        break;
                    }
                }
            }

            return result;
        }

        self.syscall_handler.execute_next_syscall(vm, hint)
    }

    // crates/blockifier/src/execution/syscalls/hint_processor.rs:280 (SyscallHintProcessor::execute_syscall)
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

    // crates/blockifier/src/execution/syscalls/hint_processor.rs:176 (verify_syscall_ptr)
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

// blockifier/src/execution/syscalls/mod.rs:222 (deploy_syscall)
fn deploy_syscall(
    request: DeployRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut CheatableSyscallHandler<'_>, // Modified parameter type
    remaining_gas: &mut u64,
) -> SyscallResult<DeployResponse> {
    // region: Modified blockifier code
    let deployer_address = syscall_handler.syscall_handler.storage_address();
    // endregion
    let deployer_address_for_calculation = if request.deploy_from_zero {
        ContractAddress::default()
    } else {
        deployer_address
    };

    let deployed_contract_address = calculate_contract_address(
        request.contract_address_salt,
        request.class_hash,
        &request.constructor_calldata,
        deployer_address_for_calculation,
    )?;

    let ctor_context = ConstructorContext {
        class_hash: request.class_hash,
        code_address: Some(deployed_contract_address),
        storage_address: deployed_contract_address,
        caller_address: deployer_address,
    };
    let call_info = execute_deployment(
        syscall_handler.syscall_handler.state,
        syscall_handler.syscall_handler.resources,
        syscall_handler.syscall_handler.context,
        ctor_context,
        request.constructor_calldata,
        *remaining_gas,
        syscall_handler.cheatnet_state,
    )?;

    let constructor_retdata = create_retdata_segment(
        vm,
        &mut syscall_handler.syscall_handler,
        &call_info.execution.retdata.0,
    )?;
    update_remaining_gas(remaining_gas, &call_info);

    syscall_handler.syscall_handler.inner_calls.push(call_info);

    Ok(DeployResponse {
        contract_address: deployed_contract_address,
        constructor_retdata,
    })
}

// blockifier/src/execution/execution_utils.rs:217 (execute_deployment)
pub fn execute_deployment(
    state: &mut dyn State,
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
    ctor_context: ConstructorContext,
    constructor_calldata: Calldata,
    remaining_gas: u64,
    cheatnet_state: &mut CheatnetState,
) -> EntryPointExecutionResult<CallInfo> {
    // Address allocation in the state is done before calling the constructor, so that it is
    // visible from it.
    let deployed_contract_address = ctor_context.storage_address;
    let current_class_hash = state.get_class_hash_at(deployed_contract_address)?;
    if current_class_hash != ClassHash::default() {
        return Err(StateError::UnavailableContractAddress(deployed_contract_address).into());
    }

    state.set_class_hash_at(deployed_contract_address, ctor_context.class_hash)?;

    let call_info = execute_constructor_entry_point(
        state,
        resources,
        context,
        ctor_context,
        constructor_calldata,
        remaining_gas,
        cheatnet_state,
    )?;

    Ok(call_info)
}

// blockifier/src/execution/syscalls/mod.rs:407 (library_call)
pub fn library_call_syscall(
    request: LibraryCallRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut CheatableSyscallHandler<'_>, // Modified parameter type
    remaining_gas: &mut u64,
) -> SyscallResult<SingleSegmentResponse> {
    let call_to_external = true;
    let retdata_segment = execute_library_call(
        syscall_handler,
        vm,
        request.class_hash,
        call_to_external,
        request.function_selector,
        request.calldata,
        remaining_gas,
    )?;

    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
}

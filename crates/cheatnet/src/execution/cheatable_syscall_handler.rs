use crate::execution::{cheated_syscalls, syscall_hooks};
use crate::state::CheatnetState;
use anyhow::Result;
use blockifier::execution::syscalls::{
    SyscallRequest, SyscallRequestWrapper, SyscallResponse, SyscallResponseWrapper, SyscallResult,
};
use blockifier::execution::{
    common_hints::HintExecutionResult,
    deprecated_syscalls::DeprecatedSyscallSelector,
    execution_utils::felt_to_stark_felt,
    syscalls::hint_processor::{SyscallExecutionError, SyscallHintProcessor},
};
use blockifier::{abi::constants, execution::syscalls::hint_processor::OUT_OF_GAS_ERROR};
use cairo_felt::Felt252;
use cairo_lang_casm::{
    hints::{Hint, StarknetHint},
    operand::{BinOpOperand, DerefOrImmediate, Operation, Register, ResOperand},
};
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::{
    types::relocatable::Relocatable,
    vm::{
        errors::{hint_errors::HintError, vm_errors::VirtualMachineError},
        vm_core::VirtualMachine,
    },
};
use starknet_api::hash::StarkFelt;
use std::any::Any;
use std::collections::HashMap;

pub type SyscallSelector = DeprecatedSyscallSelector;

// This hint processor modifies the standard syscalls implementation to react upon changes
// introduced by cheatcodes that e.g. returns mocked data
// If it cannot execute a cheatcode it falls back to SyscallHintProcessor, which provides standard implementation of
// hints from blockifier
pub struct CheatableSyscallHandler<'a, 'b>
where
    'b: 'a,
{
    pub child: &'a mut SyscallHintProcessor<'b>,
    pub cheatnet_state: &'a mut CheatnetState,
}

impl<'a, 'b> CheatableSyscallHandler<'a, 'b> {
    pub fn wrap(
        syscall_handler: &'a mut SyscallHintProcessor<'b>,
        cheatnet_state: &'a mut CheatnetState,
    ) -> Self {
        CheatableSyscallHandler {
            child: syscall_handler,
            cheatnet_state,
        }
    }
}

impl HintProcessorLogic for CheatableSyscallHandler<'_, '_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();
        if let Some(Hint::Starknet(StarknetHint::SystemCall { .. })) = maybe_extended_hint {
            if let Some(Hint::Starknet(hint)) = maybe_extended_hint {
                let selector = match self.get_syscall_selector(vm, hint) {
                    Ok(selector) => selector,
                    Err(err) => return Err(err),
                };

                let sc_res = match selector {
                    SyscallSelector::GetExecutionInfo => Some(self.execute_syscall(
                        vm,
                        cheated_syscalls::get_execution_info_syscall,
                        SyscallSelector::GetExecutionInfo,
                    )),
                    SyscallSelector::CallContract => Some(self.execute_syscall(
                        vm,
                        cheated_syscalls::call_contract_syscall,
                        SyscallSelector::CallContract,
                    )),
                    SyscallSelector::LibraryCall => Some(self.execute_syscall(
                        vm,
                        cheated_syscalls::library_call_syscall,
                        SyscallSelector::LibraryCall,
                    )),
                    SyscallSelector::Deploy => Some(self.execute_syscall(
                        vm,
                        cheated_syscalls::deploy_syscall,
                        SyscallSelector::Deploy,
                    )),
                    SyscallSelector::EmitEvent => {
                        let events_len_before = self.child.events.len();
                        let hint_exec_result = self.child.execute_next_syscall(vm, hint);
                        assert_eq!(
                            events_len_before + 1,
                            self.child.events.len(),
                            "EmitEvent syscall is expected to emit exactly one event"
                        );
                        syscall_hooks::emit_event_hook(self);
                        Some(hint_exec_result)
                    }
                    _ => None,
                };
                if let Some(sc_res) = sc_res {
                    return sc_res;
                }
            }
        }
        self.child
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.child
            .compile_hint(hint_code, ap_tracking_data, reference_ids, references)
    }
}

impl ResourceTracker for CheatableSyscallHandler<'_, '_> {
    fn consumed(&self) -> bool {
        self.child.consumed()
    }

    fn consume_step(&mut self) {
        self.child.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.child.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.child.run_resources()
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
            "snforge_std functions are not allowed in contracts".into(),
        ));
    };
    Ok(syscall)
}

fn get_syscall_cost(syscall_selector: SyscallSelector) -> u64 {
    match syscall_selector {
        SyscallSelector::LibraryCall | SyscallSelector::CallContract => {
            constants::CALL_CONTRACT_GAS_COST
        }
        SyscallSelector::Deploy => constants::DEPLOY_GAS_COST,
        SyscallSelector::GetExecutionInfo => constants::GET_EXECUTION_INFO_GAS_COST,
        _ => unreachable!("Syscall has no associated cost"),
    }
}

impl CheatableSyscallHandler<'_, '_> {
    fn get_syscall_selector(
        &self,
        vm: &mut VirtualMachine,
        hint: &StarknetHint,
    ) -> Result<SyscallSelector, HintError> {
        // We peek into the selector without incrementing the pointer as it is done later
        let syscall = get_syscall_operand(hint)?;
        let initial_syscall_ptr = get_ptr_from_res_operand_unchecked(vm, syscall);
        let selector =
            SyscallSelector::try_from(stark_felt_from_ptr_immutable(vm, &initial_syscall_ptr)?)?;

        self.verify_syscall_ptr(initial_syscall_ptr)?;
        Ok(selector)
    }

    // crates/blockifier/src/execution/syscalls/hint_processor.rs:280 (SyscallHintProcessor::execute_syscall)
    fn execute_syscall<Request, Response, ExecuteCallback>(
        &mut self,
        vm: &mut VirtualMachine,
        execute_callback: ExecuteCallback,
        selector: SyscallSelector,
    ) -> HintExecutionResult
    where
        Request: SyscallRequest + std::fmt::Debug,
        Response: SyscallResponse + std::fmt::Debug,
        ExecuteCallback: FnOnce(
            Request,
            &mut VirtualMachine,
            &mut CheatableSyscallHandler<'_, '_>,
            &mut u64, // Remaining gas.
        ) -> SyscallResult<Response>,
    {
        // Increment, since the selector was peeked into before
        self.child.syscall_ptr += 1;
        self.child.increment_syscall_count_by(&selector, 1);
        let base_gas_cost = get_syscall_cost(selector);

        let SyscallRequestWrapper {
            gas_counter,
            request,
        } = SyscallRequestWrapper::<Request>::read(vm, &mut self.child.syscall_ptr)?;

        if gas_counter < base_gas_cost {
            //  Out of gas failure.
            let out_of_gas_error =
                StarkFelt::try_from(OUT_OF_GAS_ERROR).map_err(SyscallExecutionError::from)?;
            let response: SyscallResponseWrapper<Response> = SyscallResponseWrapper::Failure {
                gas_counter,
                error_data: vec![out_of_gas_error],
            };
            response.write(vm, &mut self.child.syscall_ptr)?;

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

        response.write(vm, &mut self.child.syscall_ptr)?;

        Ok(())
    }

    // crates/blockifier/src/execution/syscalls/hint_processor.rs:176 (verify_syscall_ptr)
    fn verify_syscall_ptr(&self, actual_ptr: Relocatable) -> SyscallResult<()> {
        if actual_ptr != self.child.syscall_ptr {
            return Err(SyscallExecutionError::BadSyscallPointer {
                expected_ptr: self.child.syscall_ptr,
                actual_ptr,
            });
        }

        Ok(())
    }
}

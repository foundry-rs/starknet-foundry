use crate::execution::execution_info::get_cheated_exec_info_ptr;
use crate::execution::syscalls::cairo1_cheatable_syscall_handler::SyscallSelector;
use crate::execution::syscalls::lib::stark_felt_from_ptr_immutable;
use crate::state::CheatnetState;
use blockifier::abi::constants;
use blockifier::execution::common_hints::HintExecutionResult;
use blockifier::execution::deprecated_syscalls::hint_processor::{
    DeprecatedSyscallExecutionError, DeprecatedSyscallHintProcessor,
};
use blockifier::execution::deprecated_syscalls::{
    DeprecatedSyscallResult, DeprecatedSyscallSelector,
};
use blockifier::execution::hint_code;
use blockifier::execution::syscalls::{
    EmptyRequest, GetExecutionInfoResponse, SyscallRequest, SyscallRequestWrapper, SyscallResponse,
    SyscallResponseWrapper,
};
use cairo_felt::Felt252;
use cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_ptr_from_var_name;
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;
use std::any::Any;
use std::collections::HashMap;

pub struct Cairo0CheatableSyscallHandler<'a> {
    pub syscall_handler: DeprecatedSyscallHintProcessor<'a>,
    pub cheatnet_state: &'a mut CheatnetState,
}

impl ResourceTracker for Cairo0CheatableSyscallHandler<'_> {
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

impl HintProcessorLogic for Cairo0CheatableSyscallHandler<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> HintExecutionResult {
        let hint = hint_data
            .downcast_ref::<HintProcessorData>()
            .ok_or(HintError::WrongHintData)?;
        if hint_code::SYSCALL_HINTS.contains(hint.code.as_str()) {
            return self.execute_next_syscall_cheated(vm, &hint.ids_data, &hint.ap_tracking);
        }

        self.syscall_handler
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }
}

impl<'a> Cairo0CheatableSyscallHandler<'a> {
    pub fn verify_syscall_ptr(&self, actual_ptr: Relocatable) -> DeprecatedSyscallResult<()> {
        if actual_ptr != self.syscall_handler.syscall_ptr {
            return Err(DeprecatedSyscallExecutionError::BadSyscallPointer {
                expected_ptr: self.syscall_handler.syscall_ptr,
                actual_ptr,
            });
        }

        Ok(())
    }

    /// Infers and executes the next syscall.
    /// Must comply with the API of a hint function, as defined by the `HintProcessor`.
    pub fn execute_next_syscall_cheated(
        &mut self,
        vm: &mut VirtualMachine,
        ids_data: &HashMap<String, HintReference>,
        ap_tracking: &ApTracking,
    ) -> HintExecutionResult {
        // We peak into the selector without incrementing the pointer as it is done later
        let syscall_selector_pointer = self.syscall_handler.syscall_ptr;
        let selector = DeprecatedSyscallSelector::try_from(stark_felt_from_ptr_immutable(
            vm,
            &syscall_selector_pointer,
        )?)?;
        let contract_address = self.syscall_handler.storage_address;

        if DeprecatedSyscallSelector::GetExecutionInfo == selector
            && self.cheatnet_state.address_is_cheated(&contract_address)
        {
            let initial_syscall_ptr =
                get_ptr_from_var_name("syscall_ptr", vm, ids_data, ap_tracking)?;
            self.verify_syscall_ptr(initial_syscall_ptr)?;

            // Increment, since the selector was peeked into before
            self.syscall_handler.syscall_ptr += 1;

            if selector != DeprecatedSyscallSelector::Keccak {
                let syscall_count = self
                    .syscall_handler
                    .resources
                    .syscall_counter
                    .entry(selector)
                    .or_default();
                *syscall_count += 1;
            }

            let SyscallRequestWrapper {
                gas_counter,
                request: _,
            } = SyscallRequestWrapper::<EmptyRequest>::read(
                vm,
                &mut self.syscall_handler.syscall_ptr,
            )?;

            let execution_info_ptr = self.syscall_handler.get_or_allocate_tx_info_start_ptr(vm)?;

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
        } else if DeprecatedSyscallSelector::DelegateCall == selector {
            let dupa = "dupa";

            let x = 1;
        };

        self.syscall_handler
            .execute_next_syscall(vm, ids_data, ap_tracking)
    }

    fn execute_syscall<Request, Response, ExecuteCallback>(
        &mut self,
        vm: &mut VirtualMachine,
        execute_callback: ExecuteCallback,
    ) -> HintExecutionResult
    where
        Request: SyscallRequest,
        Response: SyscallResponse,
        ExecuteCallback: FnOnce(
            Request,
            &mut VirtualMachine,
            &mut DeprecatedSyscallHintProcessor<'_>,
        ) -> DeprecatedSyscallResult<Response>,
    {
        let request = Request::read(vm, &mut self.syscall_handler.syscall_ptr)?;

        let response = execute_callback(request, vm, &mut self.syscall_handler)?;
        response.write(vm, &mut self.syscall_handler.syscall_ptr)?;

        Ok(())
    }
}

use crate::execution::cheatable_syscall_handler::CheatableSyscallHandler;
use crate::execution::syscall_interceptor::{
    ExecuteHintRequest, HintCompilationInterceptor, HintExecutionInterceptor,
    HintProcessorExtension, HintProcessorLogicInterceptor, ResourceTrackerInterceptor,
};
use cairo_felt::Felt252;
use cairo_lang_casm::{
    hints::{Hint, StarknetHint},
    operand::ResOperand,
};
use cairo_lang_runner::{
    casm_run::{extract_relocatable, vm_get_range},
    short_string::as_cairo_short_string,
};
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};
use std::any::Any;
use std::collections::HashMap;

fn extract_input(
    vm: &mut VirtualMachine,
    input_start: &ResOperand,
    input_end: &ResOperand,
) -> Result<Vec<Felt252>, HintError> {
    let input_start = extract_relocatable(vm, input_start)?;
    let input_end = extract_relocatable(vm, input_end)?;
    vm_get_range(vm, input_start, input_end)
        .map_err(|_| HintError::CustomHint("Failed to read input data".into()))
}

pub struct ContractExecutionSyscallHandler<'a, 'b, 'c>
where
    'c: 'b,
    'b: 'a,
{
    pub child: &'a mut CheatableSyscallHandler<'b, 'c>,
}

impl<'a, 'b, 'c> ContractExecutionSyscallHandler<'a, 'b, 'c> {
    pub fn wrap(child: &'b mut CheatableSyscallHandler<'b, 'c>) -> Self {
        Self { child }
    }
}

impl HintProcessorExtension for ContractExecutionSyscallHandler<'_, '_, '_> {
    fn get_child(&self) -> Option<&dyn HintProcessorLogicInterceptor> {
        Some(self.child)
    }

    fn get_child_mut(&mut self) -> Option<&mut dyn HintProcessorLogicInterceptor> {
        Some(self.child)
    }
}
impl HintProcessorLogicInterceptor for ContractExecutionSyscallHandler<'_, '_, '_> {}
impl HintExecutionInterceptor for ContractExecutionSyscallHandler<'_, '_, '_> {
    fn intercept_execute_hint(
        &mut self,
        execute_hint_request: &mut ExecuteHintRequest,
    ) -> Option<Result<(), HintError>> {
        let maybe_extended_hint = execute_hint_request.hint_data.downcast_ref::<Hint>();

        return if let Some(Hint::Starknet(StarknetHint::Cheatcode {
            selector,
            input_start,
            input_end,
            ..
        })) = maybe_extended_hint
        {
            let selector = &selector.value.to_bytes_be().1;
            let selector = std::str::from_utf8(selector).unwrap();
            let inputs = match extract_input(execute_hint_request.vm, input_start, input_end) {
                Ok(inputs) => inputs,
                Err(err) => return Some(Err(err)),
            };

            Some(match selector {
                "print" => {
                    print(inputs);
                    Ok(())
                }
                _ => Err(HintError::CustomHint(
                    "Only `print` cheatcode is available in contracts.".into(),
                )),
            })
        } else {
            None
        };
    }
}

impl HintCompilationInterceptor for ContractExecutionSyscallHandler<'_, '_, '_> {}

impl HintProcessorLogic for ContractExecutionSyscallHandler<'_, '_, '_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        self.execute_hint_chain(vm, exec_scopes, hint_data, constants)
    }

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.compile_hint_chain(hint_code, ap_tracking_data, reference_ids, references)
    }
}
impl ResourceTrackerInterceptor for ContractExecutionSyscallHandler<'_, '_, '_> {}

impl ResourceTracker for ContractExecutionSyscallHandler<'_, '_, '_> {
    fn consumed(&self) -> bool {
        self.consumed_chain()
    }

    fn consume_step(&mut self) {
        self.consume_step_chain();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.get_n_steps_chain()
    }

    fn run_resources(&self) -> &RunResources {
        self.run_resources_chain()
    }
}

fn as_printable_short_string(value: &Felt252) -> Option<String> {
    let bytes: Vec<u8> = value.to_bytes_be();
    if bytes.iter().any(u8::is_ascii_control) {
        return None;
    }

    as_cairo_short_string(value)
}

pub fn print(inputs: Vec<Felt252>) {
    for value in inputs {
        if let Some(short_string) = as_printable_short_string(&value) {
            println!("original value: [{value}], converted to a string: [{short_string}]",);
        } else {
            println!("original value: [{value}]");
        }
    }
}

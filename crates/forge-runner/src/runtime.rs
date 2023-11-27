use std::any::Any;
use std::collections::HashMap;

use anyhow::Result;

use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;

use cairo_felt::Felt252;
use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::ResOperand;
use cairo_lang_runner::casm_run::{extract_relocatable, vm_get_range, MemBuffer};
use cairo_lang_runner::{casm_run::cell_ref_to_relocatable, insert_value_to_cellref};
use cairo_vm::hint_processor::hint_processor_definition::{
    HintProcessor, HintProcessorLogic, HintReference,
};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;

use cheatnet::cheatcodes::EnhancedHintError;

pub struct StarknetRuntime<'a> {
    pub hint_handler: SyscallHintProcessor<'a>,
}

impl<'a> ResourceTracker for StarknetRuntime<'a> {
    fn consumed(&self) -> bool {
        self.hint_handler.context.vm_run_resources.consumed()
    }

    fn consume_step(&mut self) {
        self.hint_handler.context.vm_run_resources.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.hint_handler.context.vm_run_resources.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.hint_handler.context.vm_run_resources.run_resources()
    }
}

impl<'a> HintProcessorLogic for StarknetRuntime<'a> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        self.hint_handler
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.hint_handler
            .compile_hint(hint_code, ap_tracking_data, reference_ids, references)
    }
}

pub struct RuntimeExtension<ExtensionState, Runtime: HintProcessor> {
    pub extension_state: ExtensionState,
    pub extended_runtime: Runtime,
}

pub trait RegisteredExtension: ExtensionLogic {}

// Required to implement the foreign trait
pub struct ExtendedRuntime<Extension>(pub Extension);

impl<Extension: RegisteredExtension> HintProcessorLogic for ExtendedRuntime<Extension> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();
        if let Some(Hint::Starknet(StarknetHint::Cheatcode {
            selector,
            input_start,
            input_end,
            output_start,
            output_end,
        })) = maybe_extended_hint
        {
            // Parse the selector.
            let selector = &selector.value.to_bytes_be().1;
            let selector = std::str::from_utf8(selector).map_err(|_| {
                CustomHint(Box::from(
                    "Failed to parse the  cheatcode selector".to_string(),
                ))
            })?;

            // Extract the inputs.
            let input_start = extract_relocatable(vm, input_start)?;
            let input_end = extract_relocatable(vm, input_end)?;
            let inputs = vm_get_range(vm, input_start, input_end)
                .map_err(|_| CustomHint(Box::from("Failed to read input data".to_string())))?;

            if let CheatcodeHadlingResult::Result(res) =
                self.0.handle_cheatcode(selector, inputs)?
            {
                let mut buffer = MemBuffer::new_segment(vm);
                let result_start = buffer.ptr;
                buffer
                    .write_data(res.iter())
                    .expect("Failed to insert file content to memory");
                let result_end = buffer.ptr;
                insert_value_to_cellref!(vm, output_start, result_start)?;
                insert_value_to_cellref!(vm, output_end, result_end)?;
                return Ok(());
            }
        }

        if let Some(Hint::Starknet(StarknetHint::SystemCall { system })) = maybe_extended_hint {
            // TODO move selector parsing logic here
            if let SyscallHandlingResult::Result(()) = self.0.override_system_call(system, vm)? {
                return Ok(());
            }
        }
        self.0
            .get_extended_runtime_mut()
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.0.get_extended_runtime().compile_hint(
            hint_code,
            ap_tracking_data,
            reference_ids,
            references,
        )
    }
}

impl<Handler, Runtime: HintProcessor> ResourceTracker
    for ExtendedRuntime<RuntimeExtension<Handler, Runtime>>
{
    fn consumed(&self) -> bool {
        self.0.extended_runtime.consumed()
    }

    fn consume_step(&mut self) {
        self.0.extended_runtime.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.0.extended_runtime.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.0.extended_runtime.run_resources()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum SyscallHandlingResult {
    Forward,
    Result(()),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum CheatcodeHadlingResult {
    Forward,
    Result(Vec<Felt252>),
}

pub trait ExtensionLogic {
    type Runtime: HintProcessorLogic;

    fn get_extended_runtime_mut(&mut self) -> &mut Self::Runtime;

    fn get_extended_runtime(&self) -> &Self::Runtime;

    fn override_system_call(
        &mut self,
        system: &ResOperand,
        vm: &mut VirtualMachine,
    ) -> Result<SyscallHandlingResult, HintError>;

    // TODO remove vm, output from this signature, make it return Felt252
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn handle_cheatcode(
        &mut self,
        selector: &str,
        inputs: Vec<Felt252>,
    ) -> Result<CheatcodeHadlingResult, EnhancedHintError>;
}

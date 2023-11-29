use std::any::Any;
use std::collections::HashMap;

use anyhow::Result;

use blockifier::execution::execution_utils::felt_to_stark_felt;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::execution::syscalls::SyscallResult;

use cairo_felt::Felt252;
use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_runner::casm_run::{
    extract_buffer, extract_relocatable, get_ptr, vm_get_range, MemBuffer,
};
use cairo_lang_runner::{casm_run::cell_ref_to_relocatable, insert_value_to_cellref};
use cairo_vm::hint_processor::hint_processor_definition::{
    HintProcessor, HintProcessorLogic, HintReference,
};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;

use cheatnet::cheatcodes::EnhancedHintError;
use cheatnet::execution::cheatable_syscall_handler::{CheatableSyscallHandler, SyscallSelector};

pub mod forge_runtime_extension;
pub mod io_runtime_extension;

pub trait SyscallPtrAccess {
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable;

    fn verify_syscall_ptr(&self, actual_ptr: Relocatable) -> SyscallResult<()>;
}

// TODO this is only temporary, after we migrate everything to extension it will be auto-derived
impl<'a> SyscallPtrAccess for CheatableSyscallHandler<'a> {
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable {
        &mut self.child.syscall_ptr
    }

    fn verify_syscall_ptr(&self, ptr: Relocatable) -> SyscallResult<()> {
        self.child.verify_syscall_ptr(ptr)
    }
}

pub struct StarknetRuntime<'a> {
    pub hint_handler: SyscallHintProcessor<'a>,
}

impl<'a> SyscallPtrAccess for StarknetRuntime<'a> {
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable {
        &mut self.hint_handler.syscall_ptr
    }

    fn verify_syscall_ptr(&self, ptr: Relocatable) -> SyscallResult<()> {
        self.hint_handler.verify_syscall_ptr(ptr)
    }
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

pub struct ExtendedRuntime<Extension: ExtensionLogic> {
    pub extension: Extension,
    pub extended_runtime: <Extension as ExtensionLogic>::Runtime,
}

impl<Extension: ExtensionLogic> HintProcessorLogic for ExtendedRuntime<Extension> {
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
                    "Failed to parse the cheatcode selector".to_string(),
                ))
            })?;

            // Extract the inputs.
            let input_start = extract_relocatable(vm, input_start)?;
            let input_end = extract_relocatable(vm, input_end)?;
            let inputs = vm_get_range(vm, input_start, input_end)
                .map_err(|_| CustomHint(Box::from("Failed to read input data".to_string())))?;

            if let CheatcodeHandlingResult::Handled(res) =
                self.extension
                    .handle_cheatcode(selector, inputs, &mut self.extended_runtime)?
            {
                let mut buffer = MemBuffer::new_segment(vm);
                let result_start = buffer.ptr;
                buffer
                    .write_data(res.iter())
                    .expect("Failed to insert cheatcode result to memory");
                let result_end = buffer.ptr;
                insert_value_to_cellref!(vm, output_start, result_start)?;
                insert_value_to_cellref!(vm, output_end, result_end)?;
                return Ok(());
            }
        }

        if let Some(Hint::Starknet(StarknetHint::SystemCall { system })) = maybe_extended_hint {
            let (cell, offset) = extract_buffer(system);
            let system_ptr = get_ptr(vm, cell, &offset)?;

            self.verify_syscall_ptr(system_ptr)?;

            // We peek into memory to check the selector
            let selector = SyscallSelector::try_from(felt_to_stark_felt(
                &vm.get_integer(*self.get_mut_syscall_ptr()).unwrap(),
            ))?;

            if let SyscallHandlingResult::Handled(()) =
                self.extension
                    .override_system_call(selector, vm, &mut self.extended_runtime)?
            {
                return Ok(());
            }
        }
        self.extended_runtime
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.extended_runtime
            .compile_hint(hint_code, ap_tracking_data, reference_ids, references)
    }
}

impl<Extension: ExtensionLogic> SyscallPtrAccess for ExtendedRuntime<Extension> {
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable {
        self.extended_runtime.get_mut_syscall_ptr()
    }

    fn verify_syscall_ptr(&self, ptr: Relocatable) -> SyscallResult<()> {
        self.extended_runtime.verify_syscall_ptr(ptr)
    }
}

impl<Extension: ExtensionLogic> ResourceTracker for ExtendedRuntime<Extension> {
    fn consumed(&self) -> bool {
        self.extended_runtime.consumed()
    }

    fn consume_step(&mut self) {
        self.extended_runtime.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.extended_runtime.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.extended_runtime.run_resources()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum SyscallHandlingResult {
    Forwarded,
    Handled(()),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum CheatcodeHandlingResult {
    Forwarded,
    Handled(Vec<Felt252>),
}

pub trait ExtensionLogic {
    type Runtime: HintProcessor + SyscallPtrAccess;

    fn override_system_call(
        &mut self,
        _selector: SyscallSelector,
        _vm: &mut VirtualMachine,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        Ok(SyscallHandlingResult::Forwarded)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn handle_cheatcode(
        &mut self,
        _selector: &str,
        _inputs: Vec<Felt252>,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        Ok(CheatcodeHandlingResult::Forwarded)
    }
}

use crate::execution::syscall_handler::{Passable, StackableSyscallHandler};
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;
use std::any::Any;
use std::collections::HashMap;

pub struct SyscallHandlerStack<'a> {
    syscall_handlers: Vec<&'a mut dyn StackableSyscallHandler>,
}

impl<'a> SyscallHandlerStack<'a> {
    pub fn from_handlers(syscall_handlers: Vec<&'a mut dyn StackableSyscallHandler>) -> Self {
        Self { syscall_handlers }
    }
}

impl HintProcessorLogic for SyscallHandlerStack<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        for syscall_handler in self.syscall_handlers.as_mut_slice() {
            if let Passable::Done(result) =
                syscall_handler.execute_hint(vm, exec_scopes, hint_data, constants)
            {
                return result;
            }
        }
        unreachable!("SyscallHandlerStack execute_hint fallthrough");
    }

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        for syscall_handler in self.syscall_handlers.as_slice() {
            if let Passable::Done(result) =
                syscall_handler.compile_hint(hint_code, ap_tracking_data, reference_ids, references)
            {
                return result;
            }
        }
        unreachable!("SyscallHandlerStack compile_hint fallthrough");
    }
}

impl ResourceTracker for SyscallHandlerStack<'_> {
    fn consumed(&self) -> bool {
        for syscall_handler in self.syscall_handlers.as_slice() {
            if let Passable::Done(result) = syscall_handler.consumed() {
                return result;
            }
        }
        unreachable!("SyscallHandlerStack consumed fallthrough");
    }

    fn consume_step(&mut self) {
        for mut syscall_handler in self.syscall_handlers.as_mut_slice() {
            if let Passable::Done(()) = syscall_handler.consume_step() {
                return;
            }
        }
        unreachable!("SyscallHandlerStack consume_steps fallthrough");
    }

    fn get_n_steps(&self) -> Option<usize> {
        for syscall_handler in self.syscall_handlers.as_slice() {
            if let Passable::Done(steps) = syscall_handler.get_n_steps() {
                return steps;
            }
        }
        unreachable!("SyscallHandlerStack get_n_steps fallthrough")
    }

    fn run_resources(&self) -> &RunResources {
        for syscall_handler in self.syscall_handlers.as_slice() {
            if let Passable::Done(resources) = syscall_handler.run_resources() {
                return resources;
            }
        }
        unreachable!("SyscallHandlerStack run_resources fallthrough")
    }
}

use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
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

pub enum Passable<T> {
    Pass,
    Done(T),
}

// A version of HintProcessorLogic which does not require a return of a value
pub trait StackableHintProcessorLogic {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Passable<Result<(), HintError>>;

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Passable<Result<Box<dyn Any>, VirtualMachineError>> {
        Passable::Pass
    }
}

pub trait StackableResourceTracker {
    fn consumed(&self) -> Passable<bool> {
        Passable::Pass
    }

    // Some for consumed, None for not consumed
    fn consume_step(&mut self) -> Passable<()> {
        Passable::Pass
    }

    fn get_n_steps(&self) -> Passable<Option<usize>> {
        Passable::Pass
    }

    fn run_resources(&self) -> Passable<&RunResources> {
        Passable::Pass
    }
}

pub trait StackableSyscallHandler: StackableHintProcessorLogic + StackableResourceTracker {}

impl StackableHintProcessorLogic for SyscallHintProcessor<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Passable<Result<(), HintError>> {
        Passable::Done(HintProcessorLogic::execute_hint(
            self,
            vm,
            exec_scopes,
            hint_data,
            constants,
        ))
    }
    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Passable<Result<Box<dyn Any>, VirtualMachineError>> {
        Passable::Done(HintProcessorLogic::compile_hint(
            self,
            hint_code,
            ap_tracking_data,
            reference_ids,
            references,
        ))
    }
}

impl StackableResourceTracker for SyscallHintProcessor<'_> {
    fn consumed(&self) -> Passable<bool> {
        Passable::Done(ResourceTracker::consumed(self))
    }

    // Some for consumed, None for not consumed
    fn consume_step(&mut self) -> Passable<()> {
        Passable::Done(ResourceTracker::consume_step(self))
    }

    fn get_n_steps(&self) -> Passable<Option<usize>> {
        Passable::Done(ResourceTracker::get_n_steps(self))
    }

    fn run_resources(&self) -> Passable<&RunResources> {
        Passable::Done(ResourceTracker::run_resources(self))
    }
}

impl StackableSyscallHandler for SyscallHintProcessor<'_> {}

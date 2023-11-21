use std::any::Any;
use std::collections::HashMap;

use anyhow::Result;
use blockifier::execution::common_hints::HintExecutionResult;

use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;

use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessor, HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;

use cairo_lang_casm::hints::StarknetHint;


pub struct StarknetRuntime <'a> {
    pub hint_handler: SyscallHintProcessor<'a>
}

impl<'a>  ResourceTracker for StarknetRuntime<'a> {
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
            self.hint_handler.execute_hint(vm, exec_scopes, hint_data, constants)
        }
    
        fn compile_hint(
            &self,
            hint_code: &str,
            ap_tracking_data: &ApTracking,
            reference_ids: &HashMap<String, usize>,
            references: &[HintReference],
        ) -> Result<Box<dyn Any>, VirtualMachineError> {
            self.hint_handler.compile_hint(hint_code, ap_tracking_data, reference_ids, references)
        }
}

pub struct ExtendedRuntime <Handler, Runtime: HintProcessor> {
    pub extension_handler: Handler,
    pub extended_runtime: Runtime
}

impl <Handler, Runtime: HintProcessor> ResourceTracker for ExtendedRuntime::<Handler, Runtime> {
    fn consumed(&self) -> bool {
        self.extended_runtime.consumed()
    }

    fn consume_step(&mut self) {
        self.extended_runtime.consume_step()
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.extended_runtime.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.extended_runtime.run_resources()
    }
}

impl <Handler, Runtime: HintProcessor> HintProcessorLogic for ExtendedRuntime::<Handler, Runtime> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        self.extended_runtime.execute_hint(vm, exec_scopes, hint_data, constants)
    } 

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.extended_runtime.compile_hint(hint_code, ap_tracking_data, reference_ids, references)
    }
}


enum HintHandlingResult {
    Forward,
    Result(HintExecutionResult)
}

trait Extender {
    fn handle_hint(_hint: &StarknetHint) -> HintHandlingResult {
        HintHandlingResult::Forward
    }
}


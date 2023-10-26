use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::RunResources;
use cairo_vm::vm::vm_core::VirtualMachine;
use std::any::Any;
use std::collections::HashMap;

pub struct ExecuteHintRequest<'a> {
    pub vm: &'a mut VirtualMachine,
    pub exec_scopes: &'a mut ExecutionScopes,
    pub hint_data: &'a Box<dyn Any>,
    pub constants: &'a HashMap<String, Felt252>,
}

pub trait ChainableHintProcessor {
    fn get_child(&self) -> Option<&dyn HintProcessorLogicInterceptor>;
    fn get_child_mut(&mut self) -> Option<&mut dyn HintProcessorLogicInterceptor>;
}

pub trait HintExecutionInterceptor: ChainableHintProcessor {
    fn intercept_execute_hint(
        &mut self,
        execute_hint_request: &mut ExecuteHintRequest,
    ) -> Option<Result<(), HintError>>;

    fn next_execute_hint(
        &mut self,
        execute_hint_request: &mut ExecuteHintRequest,
    ) -> Option<Result<(), HintError>> {
        if let Some(result) = self.intercept_execute_hint(execute_hint_request) {
            return Some(result);
        }
        if let Some(child) = self.get_child_mut() {
            child.next_execute_hint(execute_hint_request)
        } else {
            None
        }
    }

    fn execute_hint_chain(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        if let Some(result) = self.next_execute_hint(&mut ExecuteHintRequest {
            vm,
            exec_scopes,
            hint_data,
            constants,
        }) {
            return result;
        }
        unreachable!("execute_hint failed to terminate with a result")
    }
}

pub struct CompileHintRequest<'a> {
    pub hint_code: &'a str,
    pub ap_tracking_data: &'a ApTracking,
    pub reference_ids: &'a HashMap<String, usize>,
    pub references: &'a [HintReference],
}

pub trait HintCompilationInterceptor: ChainableHintProcessor {
    fn intercept_compile_hint(
        &self,
        _compile_hint_request: &CompileHintRequest,
    ) -> Option<Result<Box<dyn Any>, VirtualMachineError>> {
        None
    }

    fn next_compile_hint(
        &self,
        compile_hint_request: &CompileHintRequest,
    ) -> Option<Result<Box<dyn Any>, VirtualMachineError>> {
        if let Some(result) = self.intercept_compile_hint(compile_hint_request) {
            return Some(result);
        }
        if let Some(child) = self.get_child() {
            child.next_compile_hint(compile_hint_request)
        } else {
            None
        }
    }
    fn compile_hint_chain(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        if let Some(result) = self.next_compile_hint(&CompileHintRequest {
            hint_code,
            ap_tracking_data,
            reference_ids,
            references,
        }) {
            return result;
        }
        unreachable!("compile_hint failed to terminate with a result")
    }
}

pub trait ResourceTrackerInterceptor: ChainableHintProcessor {
    fn intercept_consumed(&self) -> Option<bool> {
        None
    }

    fn next_consumed(&self) -> Option<bool> {
        if let Some(result) = self.intercept_consumed() {
            return Some(result);
        }
        if let Some(child) = self.get_child() {
            child.next_consumed()
        } else {
            None
        }
    }

    fn consumed_chain(&self) -> bool {
        if let Some(result) = self.next_consumed() {
            return result;
        }
        unreachable!("consumed_chain failed to terminate with a result")
    }

    // consume_step
    fn intercept_consume_step(&mut self) -> Option<()> {
        None
    }

    fn next_consume_step(&mut self) -> Option<()> {
        if let Some(result) = self.intercept_consume_step() {
            return Some(result);
        }
        if let Some(child) = self.get_child_mut() {
            child.next_consume_step()
        } else {
            None
        }
    }

    fn consume_step_chain(&mut self) {
        if let Some(result) = self.next_consume_step() {
            return result;
        }
        unreachable!("consume_step_chain failed to terminate with a result")
    }

    // get_n_steps
    fn intercept_get_n_steps(&self) -> Option<Option<usize>> {
        None
    }

    fn next_get_n_steps(&self) -> Option<Option<usize>> {
        if let Some(result) = self.intercept_get_n_steps() {
            return Some(result);
        }
        if let Some(child) = self.get_child() {
            child.next_get_n_steps()
        } else {
            None
        }
    }

    fn get_n_steps_chain(&self) -> Option<usize> {
        if let Some(result) = self.next_get_n_steps() {
            return result;
        }
        unreachable!("get_n_steps_chain failed to terminate with a result")
    }

    // run_resources
    fn intercept_run_resources(&self) -> Option<&RunResources> {
        None
    }

    fn next_run_resources(&self) -> Option<&RunResources> {
        if let Some(result) = self.intercept_run_resources() {
            return Some(result);
        }
        if let Some(child) = self.get_child() {
            child.next_run_resources()
        } else {
            None
        }
    }

    fn run_resources_chain(&self) -> &RunResources {
        if let Some(result) = self.next_run_resources() {
            return result;
        }
        unreachable!("run_resources_chain failed to terminate with a result")
    }
}

pub trait HintProcessorLogicInterceptor:
    HintExecutionInterceptor + HintCompilationInterceptor + ResourceTrackerInterceptor
{
}

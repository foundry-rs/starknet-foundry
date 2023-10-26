use crate::execution::syscall_interceptor::{
    ChainableHintProcessor, CompileHintRequest, ExecuteHintRequest, HintCompilationInterceptor,
    HintExecutionInterceptor, HintProcessorLogicInterceptor, ResourceTrackerInterceptor,
};
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessorLogic;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use std::any::Any;

impl ChainableHintProcessor for SyscallHintProcessor<'_> {
    fn get_child(&self) -> Option<&dyn HintProcessorLogicInterceptor> {
        None
    }

    fn get_child_mut(&mut self) -> Option<&mut dyn HintProcessorLogicInterceptor> {
        None
    }
}

impl HintExecutionInterceptor for SyscallHintProcessor<'_> {
    fn intercept_execute_hint(
        &mut self,
        execute_hint_request: &mut ExecuteHintRequest,
    ) -> Option<Result<(), HintError>> {
        Some(HintProcessorLogic::execute_hint(
            self,
            execute_hint_request.vm,
            execute_hint_request.exec_scopes,
            execute_hint_request.hint_data,
            execute_hint_request.constants,
        ))
    }
}

impl HintCompilationInterceptor for SyscallHintProcessor<'_> {
    fn intercept_compile_hint(
        &self,
        compile_hint_request: &CompileHintRequest,
    ) -> Option<Result<Box<dyn Any>, VirtualMachineError>> {
        Some(HintProcessorLogic::compile_hint(
            self,
            compile_hint_request.hint_code,
            compile_hint_request.ap_tracking_data,
            compile_hint_request.reference_ids,
            compile_hint_request.references,
        ))
    }
}

impl ResourceTrackerInterceptor for SyscallHintProcessor<'_> {
    fn intercept_consumed(&self) -> Option<bool> {
        Some(ResourceTracker::consumed(self))
    }
    fn intercept_consume_step(&mut self) -> Option<()> {
        ResourceTracker::consume_step(self);
        Some(())
    }
    fn intercept_get_n_steps(&self) -> Option<Option<usize>> {
        Some(ResourceTracker::get_n_steps(self))
    }
    fn intercept_run_resources(&self) -> Option<&RunResources> {
        Some(ResourceTracker::run_resources(self))
    }
}

impl HintProcessorLogicInterceptor for SyscallHintProcessor<'_> {}

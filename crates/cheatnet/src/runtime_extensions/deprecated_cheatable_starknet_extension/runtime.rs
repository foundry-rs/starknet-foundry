use crate::runtime_extensions::cheatable_starknet_runtime_extension::felt_from_ptr_immutable;
use anyhow::Result;
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallResult;
use blockifier::execution::{
    deprecated_syscalls::{
        hint_processor::DeprecatedSyscallHintProcessor, DeprecatedSyscallSelector,
    },
    hint_code,
    syscalls::{hint_processor::SyscallExecutionError, SyscallResult},
};
use cairo_vm::{
    hint_processor::{
        builtin_hint_processor::{
            builtin_hint_processor_definition::HintProcessorData, hint_utils::get_ptr_from_var_name,
        },
        hint_processor_definition::{HintProcessor, HintProcessorLogic, HintReference},
    },
    serde::deserialize_program::ApTracking,
    types::{exec_scope::ExecutionScopes, relocatable::Relocatable},
    vm::{
        errors::{hint_errors::HintError, vm_errors::VirtualMachineError},
        runners::cairo_runner::{ResourceTracker, RunResources},
        vm_core::VirtualMachine,
    },
};
use runtime::{SyscallHandlingResult, SyscallPtrAccess};
use starknet_types_core::felt::Felt;
use std::{any::Any, collections::HashMap};

pub struct DeprecatedStarknetRuntime<'a> {
    pub hint_handler: DeprecatedSyscallHintProcessor<'a>,
}

impl SyscallPtrAccess for DeprecatedStarknetRuntime<'_> {
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable {
        &mut self.hint_handler.syscall_ptr
    }
}

impl ResourceTracker for DeprecatedStarknetRuntime<'_> {
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

impl HintProcessorLogic for DeprecatedStarknetRuntime<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt>,
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

pub struct DeprecatedExtendedRuntime<Extension: DeprecatedExtensionLogic> {
    pub extension: Extension,
    pub extended_runtime: <Extension as DeprecatedExtensionLogic>::Runtime,
}

impl<Extension: DeprecatedExtensionLogic> HintProcessorLogic
    for DeprecatedExtendedRuntime<Extension>
{
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt>,
    ) -> Result<(), HintError> {
        let hint = hint_data
            .downcast_ref::<HintProcessorData>()
            .ok_or(HintError::WrongHintData)?;
        if hint_code::SYSCALL_HINTS.contains(hint.code.as_str()) {
            return self.execute_syscall_hint(
                vm,
                exec_scopes,
                hint_data,
                &hint.ids_data,
                constants,
                &hint.ap_tracking,
            );
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

impl<Extension: DeprecatedExtensionLogic> DeprecatedExtendedRuntime<Extension> {
    #[allow(clippy::too_many_arguments)]
    fn execute_syscall_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        ids_data: &HashMap<String, HintReference>,
        constants: &HashMap<String, Felt>,
        ap_tracking: &ApTracking,
    ) -> Result<(), HintError> {
        let initial_syscall_ptr = get_ptr_from_var_name("syscall_ptr", vm, ids_data, ap_tracking)?;
        // self.verify_syscall_ptr(initial_syscall_ptr)?;
        let selector = DeprecatedSyscallSelector::try_from(felt_from_ptr_immutable(
            vm,
            &initial_syscall_ptr,
        )?)?;

        if let SyscallHandlingResult::Handled =
            self.extension
                .override_system_call(selector, vm, &mut self.extended_runtime)?
        {
            Ok(())
        } else {
            self.extended_runtime
                .execute_hint(vm, exec_scopes, hint_data, constants)?;

            self.extension
                .post_syscall_hook(&selector, &mut self.extended_runtime);

            Ok(())
        }
    }
}

impl<Extension: DeprecatedExtensionLogic> SyscallPtrAccess
    for DeprecatedExtendedRuntime<Extension>
{
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable {
        self.extended_runtime.get_mut_syscall_ptr()
    }

    // fn verify_syscall_ptr(&self, ptr: Relocatable) -> SyscallResult<()> {
    //     self.extended_runtime.verify_syscall_ptr(ptr)
    // }
}

impl<Extension: DeprecatedExtensionLogic> ResourceTracker for DeprecatedExtendedRuntime<Extension> {
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

pub trait DeprecatedExtensionLogic {
    type Runtime: HintProcessor + SyscallPtrAccess;

    fn override_system_call(
        &mut self,
        _selector: DeprecatedSyscallSelector,
        _vm: &mut VirtualMachine,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        Ok(SyscallHandlingResult::Forwarded)
    }

    fn post_syscall_hook(
        &mut self,
        _selector: &DeprecatedSyscallSelector,
        _extended_runtime: &mut Self::Runtime,
    );
}

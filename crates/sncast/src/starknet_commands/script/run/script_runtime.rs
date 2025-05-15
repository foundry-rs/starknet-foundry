use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use cairo_lang_casm::hints::{ExternalHint, Hint};
use cairo_lang_runner::Arg;
use cairo_lang_runner::casm_run::{cell_ref_to_relocatable, extract_relocatable, get_val};
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;
use num_traits::ToPrimitive;
use runtime::{SignalPropagator, StarknetRuntime, SyscallPtrAccess};
use starknet_types_core::felt::Felt;
use std::any::Any;
use std::collections::HashMap;

pub struct CastScriptRuntime<'a> {
    pub starknet_runtime: StarknetRuntime<'a>,
    pub user_args: Vec<Vec<Arg>>,
}

impl SyscallPtrAccess for CastScriptRuntime<'_> {
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable {
        self.starknet_runtime.get_mut_syscall_ptr()
    }
}

impl ResourceTracker for CastScriptRuntime<'_> {
    fn consumed(&self) -> bool {
        self.starknet_runtime.consumed()
    }

    fn consume_step(&mut self) {
        self.starknet_runtime.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.starknet_runtime.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.starknet_runtime.run_resources()
    }
}

impl SignalPropagator for CastScriptRuntime<'_> {
    fn propagate_system_call_signal(
        &mut self,
        selector: DeprecatedSyscallSelector,
        vm: &mut VirtualMachine,
    ) {
        self.starknet_runtime
            .propagate_system_call_signal(selector, vm);
    }

    fn propagate_cheatcode_signal(&mut self, selector: &str, inputs: &[Felt]) {
        self.starknet_runtime
            .propagate_cheatcode_signal(selector, inputs);
    }
}

impl HintProcessorLogic for CastScriptRuntime<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt>,
    ) -> Result<(), HintError> {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();

        match maybe_extended_hint {
            // Copied from https://github.com/starkware-libs/cairo/blob/ada0450439a36d756223fba88dfd1f266f428f0c/crates/cairo-lang-runner/src/casm_run/mod.rs#L1321
            Some(Hint::External(ExternalHint::AddRelocationRule { src, dst })) => {
                vm.add_relocation_rule(
                    extract_relocatable(vm, src)?,
                    extract_relocatable(vm, dst)?,
                )?;
                Ok(())
            }
            // Copied from https://github.com/starkware-libs/cairo/blob/ada0450439a36d756223fba88dfd1f266f428f0c/crates/cairo-lang-runner/src/casm_run/mod.rs#L1330
            Some(Hint::External(ExternalHint::WriteRunParam { index, dst })) => {
                let index = get_val(vm, index)?.to_usize().expect("Got a bad index.");
                let mut stack = vec![(cell_ref_to_relocatable(dst, vm), &self.user_args[index])];
                while let Some((mut buffer, values)) = stack.pop() {
                    for value in values {
                        match value {
                            Arg::Value(v) => {
                                vm.insert_value(buffer, v)?;
                                buffer += 1;
                            }
                            Arg::Array(arr) => {
                                let arr_buffer = vm.add_memory_segment();
                                stack.push((arr_buffer, arr));
                                vm.insert_value(buffer, arr_buffer)?;
                                buffer += 1;
                                vm.insert_value(buffer, (arr_buffer + args_size(arr))?)?;
                                buffer += 1;
                            }
                        }
                    }
                }
                Ok(())
            }
            _ => self
                .starknet_runtime
                .execute_hint(vm, exec_scopes, hint_data, constants),
        }
    }

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.starknet_runtime
            .compile_hint(hint_code, ap_tracking_data, reference_ids, references)
    }
}

fn args_size(args: &[Arg]) -> usize {
    args.iter().map(Arg::size).sum()
}

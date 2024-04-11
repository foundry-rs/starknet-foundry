use anyhow::Result;
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::execution_utils::felt_to_stark_felt;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::execution::syscalls::SyscallResult;
use blockifier::state::errors::StateError;
use cairo_felt::Felt252;
use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_runner::casm_run::{
    extract_buffer, extract_relocatable, get_ptr, vm_get_range, MemBuffer,
};
use cairo_lang_runner::{casm_run::cell_ref_to_relocatable, insert_value_to_cellref};
use cairo_lang_utils::bigint::BigIntAsHex;
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
use conversions::byte_array::ByteArray;
use conversions::felt252::SerializeAsFelt252Vec;
use starknet_api::StarknetApiError;
use std::any::Any;
use std::collections::HashMap;
use std::io;
use thiserror::Error;
use utils::BufferReader;

pub mod starknet;
pub mod utils;

// from core/src/starknet/testing.cairo
const CAIRO_TEST_CHEATCODES: [&str; 14] = [
    "set_block_number",
    "set_caller_address",
    "set_contract_address",
    "set_sequencer_address",
    "set_block_timestamp",
    "set_version",
    "set_account_contract_address",
    "set_max_fee",
    "set_transaction_hash",
    "set_chain_id",
    "set_nonce",
    "set_signature",
    "pop_log",
    "pop_l2_to_l1_message",
];
pub trait SyscallPtrAccess {
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable;

    fn verify_syscall_ptr(&self, actual_ptr: Relocatable) -> SyscallResult<()>;
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

impl<'a> SignalPropagator for StarknetRuntime<'a> {
    fn propagate_system_call_signal(
        &mut self,
        _selector: DeprecatedSyscallSelector,
        _vm: &mut VirtualMachine,
    ) {
    }

    fn propagate_cheatcode_signal(&mut self, _selector: &str, _inputs: &[Felt252]) {}
}

pub fn parse_selector(selector: &BigIntAsHex) -> Result<String, HintError> {
    let selector = &selector.value.to_bytes_be().1;
    let selector = std::str::from_utf8(selector)
        .map_err(|_| CustomHint(Box::from("Failed to parse the cheatcode selector")))?;
    Ok(String::from(selector))
}

fn fetch_cheatcode_input(
    vm: &mut VirtualMachine,
    input_start: &ResOperand,
    input_end: &ResOperand,
) -> Result<Vec<Felt252>, HintError> {
    let input_start = extract_relocatable(vm, input_start)?;
    let input_end = extract_relocatable(vm, input_end)?;
    let inputs = vm_get_range(vm, input_start, input_end)
        .map_err(|_| CustomHint(Box::from("Failed to read input data")))?;
    Ok(inputs)
}

impl<'a> HintProcessorLogic for StarknetRuntime<'a> {
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
            input_start: _,
            input_end: _,
            output_start: _,
            output_end: _,
        })) = maybe_extended_hint
        {
            let selector = parse_selector(selector)?;

            let is_cairo_test_fn = CAIRO_TEST_CHEATCODES.contains(&selector.as_str());

            let error = format!(
                "Function `{selector}` is not supported in this runtime\n{}",
                if is_cairo_test_fn {
                    "Check if functions are imported from `snforge_std`/`sncast_std` NOT from `starknet::testing`"
                } else {
                    "Check if used library (`snforge_std` or `sncast_std`) is compatible with used binary, probably one of them is not updated"
                }
            );

            return Err(HintError::CustomHint(error.into()));
        }

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
        if let Some(Hint::Starknet(starknet_hint)) = maybe_extended_hint {
            if let StarknetHint::Cheatcode {
                selector,
                input_start,
                input_end,
                output_start,
                output_end,
            } = starknet_hint
            {
                return self.execute_cheatcode_hint(
                    vm,
                    exec_scopes,
                    hint_data,
                    constants,
                    selector,
                    &VmIoPointers {
                        input_start,
                        input_end,
                        output_start,
                        output_end,
                    },
                );
            } else if let StarknetHint::SystemCall { system } = starknet_hint {
                return self.execute_syscall_hint(vm, exec_scopes, hint_data, constants, system);
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

struct VmIoPointers<'a> {
    input_start: &'a ResOperand,
    input_end: &'a ResOperand,
    output_start: &'a CellRef,
    output_end: &'a CellRef,
}

impl<Extension: ExtensionLogic> ExtendedRuntime<Extension> {
    fn execute_cheatcode_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
        selector: &BigIntAsHex,
        vm_io_ptrs: &VmIoPointers,
    ) -> Result<(), HintError> {
        let selector = parse_selector(selector)?;
        let inputs = fetch_cheatcode_input(vm, vm_io_ptrs.input_start, vm_io_ptrs.input_end)?;

        let result = self.extension.handle_cheatcode(
            &selector,
            BufferReader::new(&inputs),
            &mut self.extended_runtime,
        );

        let res = match result {
            Ok(CheatcodeHandlingResult::Forwarded) => {
                let res = self
                    .extended_runtime
                    .execute_hint(vm, exec_scopes, hint_data, constants);
                self.extension.handle_cheatcode_signal(
                    &selector,
                    &inputs,
                    &mut self.extended_runtime,
                );
                return res;
            }
            Ok(CheatcodeHandlingResult::Handled(res)) => Ok(res),
            Err(err) => Err(ByteArray::from(err.to_string().as_str()).serialize_no_magic()),
        }
        .serialize_as_felt252_vec();

        let mut buffer = MemBuffer::new_segment(vm);
        let result_start = buffer.ptr;
        buffer
            .write_data(res.iter())
            .expect("Failed to insert cheatcode result to memory");
        let result_end = buffer.ptr;
        let output_start = vm_io_ptrs.output_start;
        let output_end = vm_io_ptrs.output_end;

        insert_value_to_cellref!(vm, output_start, result_start)?;
        insert_value_to_cellref!(vm, output_end, result_end)?;

        self.propagate_cheatcode_signal(&selector, &inputs);

        Ok(())
    }
    fn execute_syscall_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
        system: &ResOperand,
    ) -> Result<(), HintError> {
        let (cell, offset) = extract_buffer(system);
        let system_ptr = get_ptr(vm, cell, &offset)?;

        self.verify_syscall_ptr(system_ptr)?;

        // We peek into memory to check the selector
        let selector = DeprecatedSyscallSelector::try_from(felt_to_stark_felt(
            &vm.get_integer(*self.get_mut_syscall_ptr()).unwrap(),
        ))?;

        if let SyscallHandlingResult::Handled(()) =
            self.extension
                .override_system_call(selector, vm, &mut self.extended_runtime)?
        {
            self.propagate_system_call_signal(selector, vm);
            Ok(())
        } else {
            self.extended_runtime
                .execute_hint(vm, exec_scopes, hint_data, constants)?;
            self.extension
                .handle_system_call_signal(selector, vm, &mut self.extended_runtime);
            Ok(())
        }
    }
}

pub trait SignalPropagator {
    fn propagate_system_call_signal(
        &mut self,
        selector: DeprecatedSyscallSelector,
        vm: &mut VirtualMachine,
    );

    fn propagate_cheatcode_signal(&mut self, selector: &str, inputs: &[Felt252]);
}

impl<Extension: ExtensionLogic> SignalPropagator for ExtendedRuntime<Extension> {
    fn propagate_system_call_signal(
        &mut self,
        selector: DeprecatedSyscallSelector,
        vm: &mut VirtualMachine,
    ) {
        self.extended_runtime
            .propagate_system_call_signal(selector, vm);
        self.extension
            .handle_system_call_signal(selector, vm, &mut self.extended_runtime);
    }

    fn propagate_cheatcode_signal(&mut self, selector: &str, inputs: &[Felt252]) {
        self.extended_runtime
            .propagate_cheatcode_signal(selector, inputs);
        self.extension
            .handle_cheatcode_signal(selector, inputs, &mut self.extended_runtime);
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

impl CheatcodeHandlingResult {
    pub fn from_serializable(serializable: impl SerializeAsFelt252Vec) -> Self {
        Self::Handled(serializable.serialize_as_felt252_vec())
    }
}

pub trait ExtensionLogic {
    type Runtime: HintProcessor + SyscallPtrAccess + SignalPropagator;

    fn override_system_call(
        &mut self,
        _selector: DeprecatedSyscallSelector,
        _vm: &mut VirtualMachine,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        Ok(SyscallHandlingResult::Forwarded)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn handle_cheatcode(
        &mut self,
        _selector: &str,
        _input_reader: BufferReader,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        Ok(CheatcodeHandlingResult::Forwarded)
    }

    /// Different from `override_system_call` because it cannot be overridden,
    /// always receives a signal and cannot return an error
    /// Signals are executed in reverse order to normal syscall handlers
    /// Signals are executed after syscall is handled
    fn handle_system_call_signal(
        &mut self,
        _selector: DeprecatedSyscallSelector,
        _vm: &mut VirtualMachine,
        _extended_runtime: &mut Self::Runtime,
    ) {
    }

    /// Different from `handle_cheadcode` because it cannot be overridden,
    /// always receives a signal and cannot return an error
    /// Signals are executed in reverse order to normal cheatcode handlers
    /// Signals are executed after cheatcode is handled
    fn handle_cheatcode_signal(
        &mut self,
        _selector: &str,
        _inputs: &[Felt252],
        _extended_runtime: &mut Self::Runtime,
    ) {
    }
}

// All errors that can be thrown from the hint executor have to be added here,
// to prevent the whole runner from panicking
#[derive(Error, Debug)]
pub enum EnhancedHintError {
    #[error(transparent)]
    Hint(#[from] HintError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    State(#[from] StateError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StarknetApi(#[from] StarknetApiError),
    #[error("Failed to parse {path} file")]
    FileParsing { path: String },
}

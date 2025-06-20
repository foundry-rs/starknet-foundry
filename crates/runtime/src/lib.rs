use crate::vm::{cell_ref_to_relocatable, extract_relocatable, get_val, vm_get_range};
use anyhow::Result;
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::errors::StateError;
use cairo_lang_casm::hints::{ExternalHint, Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
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
use conversions::serde::deserialize::BufferReadError;
use conversions::serde::deserialize::BufferReader;
use conversions::serde::serialize::raw::RawFeltVec;
use conversions::serde::serialize::{CairoSerialize, SerializeToFeltVec};
use indoc::indoc;
use shared::vm::VirtualMachineExt;
use starknet_api::StarknetApiError;
use starknet_types_core::felt::Felt;
use std::any::Any;
use std::collections::HashMap;
use std::io;
use thiserror::Error;

pub mod starknet;
mod vm;

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
}

pub struct StarknetRuntime<'a> {
    pub hint_handler: SyscallHintProcessor<'a>,
    pub panic_traceback: Option<Vec<usize>>,
}

impl SyscallPtrAccess for StarknetRuntime<'_> {
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable {
        &mut self.hint_handler.syscall_ptr
    }
}

impl ResourceTracker for StarknetRuntime<'_> {
    fn consumed(&self) -> bool {
        self.hint_handler.base.context.vm_run_resources.consumed()
    }

    fn consume_step(&mut self) {
        self.hint_handler
            .base
            .context
            .vm_run_resources
            .consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.hint_handler
            .base
            .context
            .vm_run_resources
            .get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.hint_handler
            .base
            .context
            .vm_run_resources
            .run_resources()
    }
}

impl SignalPropagator for StarknetRuntime<'_> {
    fn propagate_system_call_signal(
        &mut self,
        _selector: DeprecatedSyscallSelector,
        _vm: &mut VirtualMachine,
    ) {
    }

    fn propagate_cheatcode_signal(&mut self, _selector: &str, _inputs: &[Felt]) {}
}

fn parse_selector(selector: &BigIntAsHex) -> Result<String, HintError> {
    let selector = &selector.value.to_bytes_be().1;
    let selector = std::str::from_utf8(selector)
        .map_err(|_| CustomHint(Box::from("Failed to parse the cheatcode selector")))?;
    Ok(String::from(selector))
}

fn fetch_cheatcode_input(
    vm: &mut VirtualMachine,
    input_start: &ResOperand,
    input_end: &ResOperand,
) -> Result<Vec<Felt>, HintError> {
    let input_start = extract_relocatable(vm, input_start)?;
    let input_end = extract_relocatable(vm, input_end)?;
    let inputs = vm_get_range(vm, input_start, input_end)
        .map_err(|_| CustomHint(Box::from("Failed to read input data")))?;
    Ok(inputs)
}

impl HintProcessorLogic for StarknetRuntime<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt>,
    ) -> Result<(), HintError> {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();

        if let Some(extended_hint) = maybe_extended_hint {
            match extended_hint {
                Hint::Starknet(StarknetHint::Cheatcode {
                    selector,
                    input_start: _,
                    input_end: _,
                    output_start: _,
                    output_end: _,
                }) => {
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

                    return Err(CustomHint(error.into()));
                }
                Hint::External(ExternalHint::AddTrace { flag }) => {
                    const PANIC_IN_BYTES: Felt = Felt::from_hex_unchecked("0x70616e6963");
                    let flag = get_val(vm, flag)?;
                    // Setting the panic backtrace if the given flag is panic.
                    if flag == PANIC_IN_BYTES {
                        self.panic_traceback = Some(vm.get_reversed_pc_traceback());
                    }
                    return Ok(());
                }
                _ => {}
            }
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
        constants: &HashMap<String, Felt>,
    ) -> Result<(), HintError> {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();

        match maybe_extended_hint {
            Some(Hint::Starknet(starknet_hint)) => match starknet_hint {
                StarknetHint::Cheatcode {
                    selector,
                    input_start,
                    input_end,
                    output_start,
                    output_end,
                } => self.execute_cheatcode_hint(
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
                ),
                StarknetHint::SystemCall { .. } => {
                    self.execute_syscall_hint(vm, exec_scopes, hint_data, constants)
                }
            },
            _ => self
                .extended_runtime
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
        constants: &HashMap<String, Felt>,
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
            // it is serialized again to add `Result` discriminator
            Ok(CheatcodeHandlingResult::Handled(res)) => Ok(RawFeltVec::new(res)),
            Err(err) => Err(ByteArray::from(err.to_string().as_str())),
        }
        .serialize_to_vec();

        let WrittenData {
            start: result_start,
            end: result_end,
        } = write_data(res, vm)?;
        let output_start = vm_io_ptrs.output_start;
        let output_end = vm_io_ptrs.output_end;

        vm.insert_value(cell_ref_to_relocatable(*output_start, vm), result_start)?;
        vm.insert_value(cell_ref_to_relocatable(*output_end, vm), result_end)?;

        self.propagate_cheatcode_signal(&selector, &inputs);

        Ok(())
    }
    fn execute_syscall_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt>,
    ) -> Result<(), HintError> {
        // We peek into memory to check the selector
        let selector = DeprecatedSyscallSelector::try_from(
            vm.get_integer(*self.get_mut_syscall_ptr())
                .unwrap()
                .into_owned(),
        )?;

        if let SyscallHandlingResult::Handled =
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

    fn propagate_cheatcode_signal(&mut self, selector: &str, inputs: &[Felt]);
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

    fn propagate_cheatcode_signal(&mut self, selector: &str, inputs: &[Felt]) {
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

#[derive(Debug)]
pub enum SyscallHandlingResult {
    Forwarded,
    Handled,
}

#[derive(Debug)]
pub enum CheatcodeHandlingResult {
    Forwarded,
    Handled(Vec<Felt>),
}

impl CheatcodeHandlingResult {
    pub fn from_serializable(serializable: impl CairoSerialize) -> Self {
        Self::Handled(serializable.serialize_to_vec())
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
        _inputs: &[Felt],
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

impl From<BufferReadError> for EnhancedHintError {
    fn from(value: BufferReadError) -> Self {
        EnhancedHintError::Anyhow(
                anyhow::Error::from(value)
                    .context(
                        indoc!(r"
                        Reading from buffer failed, this can be caused by calling starknet::testing::cheatcode with invalid arguments.
                        Probably `snforge_std`/`sncast_std` version is incompatible, check above for incompatibility warning.
                    ")
                    )
            )
    }
}

struct WrittenData {
    start: Relocatable,
    end: Relocatable,
}

fn write_data(data: Vec<Felt>, vm: &mut VirtualMachine) -> Result<WrittenData, HintError> {
    let mut ptr = vm.add_memory_segment();
    let start = ptr;
    for data in data {
        vm.insert_value(ptr, data)?;
        ptr += 1;
    }
    Ok(WrittenData { start, end: ptr })
}

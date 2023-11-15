use anyhow::Result;
use blockifier::execution::{
    execution_utils::felt_to_stark_felt, syscalls::hint_processor::SyscallHintProcessor,
};
use cairo_felt::Felt252;
use cairo_lang_casm::operand::ResOperand;
use cairo_lang_casm::operand::{BinOpOperand, DerefOrImmediate, Operation, Register};
use cairo_lang_runner::casm_run::{extract_relocatable, vm_get_range};
use cairo_lang_utils::bigint::BigIntAsHex;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};
use starknet_api::hash::StarkFelt;

pub fn get_syscall_selector(
    syscall: &ResOperand,
    vm: &mut VirtualMachine,
    syscall_handler: &SyscallHintProcessor<'_>,
) -> Result<StarkFelt, HintError> {
    let initial_syscall_ptr = get_ptr_from_res_operand_unchecked(vm, syscall);
    syscall_handler.verify_syscall_ptr(initial_syscall_ptr)?;
    let selector = stark_felt_from_ptr_immutable(vm, &initial_syscall_ptr)?;
    Ok(selector)
}

pub fn stark_felt_from_ptr_immutable(
    vm: &VirtualMachine,
    ptr: &Relocatable,
) -> Result<StarkFelt, VirtualMachineError> {
    Ok(felt_to_stark_felt(&felt_from_ptr_immutable(vm, ptr)?))
}

pub fn felt_from_ptr_immutable(
    vm: &VirtualMachine,
    ptr: &Relocatable,
) -> Result<Felt252, VirtualMachineError> {
    let felt = vm.get_integer(*ptr)?.into_owned();
    Ok(felt)
}

pub fn parse_selector(selector: &BigIntAsHex) -> Result<String, HintError> {
    let selector = selector.value.to_bytes_be().1;
    String::from_utf8(selector)
        .map_err(|_| HintError::CustomHint("Failed to parse selector".to_string().into()))
}

pub fn extract_input(
    vm: &mut VirtualMachine,
    input_start: &ResOperand,
    input_end: &ResOperand,
) -> Result<Vec<Felt252>, HintError> {
    let input_start = extract_relocatable(vm, input_start)?;
    let input_end = extract_relocatable(vm, input_end)?;
    vm_get_range(vm, input_start, input_end)
        .map_err(|_| HintError::CustomHint("Failed to read input data".into()))
}

// crates/blockifier/src/execution/syscalls/hint_processor.rs:454
/// Retrieves a [Relocatable] from the VM given a [`ResOperand`].
/// A [`ResOperand`] represents a CASM result expression, and is deserialized with the hint.
pub fn get_ptr_from_res_operand_unchecked(
    vm: &mut VirtualMachine,
    res: &ResOperand,
) -> Relocatable {
    let (cell, base_offset) = match res {
        ResOperand::Deref(cell) => (cell, Felt252::from(0)),
        ResOperand::BinOp(BinOpOperand {
            op: Operation::Add,
            a,
            b: DerefOrImmediate::Immediate(b),
        }) => (a, Felt252::from(b.clone().value)),
        _ => panic!("Illegal argument for a buffer."),
    };
    let base = match cell.register {
        Register::AP => vm.get_ap(),
        Register::FP => vm.get_fp(),
    };
    let cell_reloc = (base + i32::from(cell.offset)).unwrap();
    (vm.get_relocatable(cell_reloc).unwrap() + &base_offset).unwrap()
}

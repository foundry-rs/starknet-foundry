//! Contains code copied from `cairo-lang-runner`
use cairo_lang_casm::operand::{
    BinOpOperand, CellRef, DerefOrImmediate, Operation, Register, ResOperand,
};
use cairo_lang_utils::extract_matches;
use cairo_vm::Felt252;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::vm_core::VirtualMachine;

pub fn cell_ref_to_relocatable(cell_ref: CellRef, vm: &VirtualMachine) -> Relocatable {
    let base = match cell_ref.register {
        Register::AP => vm.get_ap(),
        Register::FP => vm.get_fp(),
    };
    (base + i32::from(cell_ref.offset)).unwrap()
}

/// Extracts a parameter assumed to be a buffer.
fn extract_buffer(buffer: &ResOperand) -> (CellRef, Felt252) {
    let (cell, base_offset) = match buffer {
        ResOperand::Deref(cell) => (cell, 0.into()),
        ResOperand::BinOp(BinOpOperand {
            op: Operation::Add,
            a,
            b,
        }) => (
            a,
            extract_matches!(b, DerefOrImmediate::Immediate)
                .clone()
                .value
                .into(),
        ),
        _ => panic!("Illegal argument for a buffer."),
    };
    (*cell, base_offset)
}

/// Fetches the value of a cell plus an offset from the vm, useful for pointers.
fn get_ptr(
    vm: &VirtualMachine,
    cell: CellRef,
    offset: &Felt252,
) -> Result<Relocatable, VirtualMachineError> {
    Ok((vm.get_relocatable(cell_ref_to_relocatable(cell, vm))? + offset)?)
}

/// Extracts a parameter assumed to be a buffer, and converts it into a relocatable.
pub fn extract_relocatable(
    vm: &VirtualMachine,
    buffer: &ResOperand,
) -> Result<Relocatable, VirtualMachineError> {
    let (base, offset) = extract_buffer(buffer);
    get_ptr(vm, base, &offset)
}

pub fn vm_get_range(
    vm: &mut VirtualMachine,
    mut calldata_start_ptr: Relocatable,
    calldata_end_ptr: Relocatable,
) -> Result<Vec<Felt252>, HintError> {
    let mut values = vec![];
    while calldata_start_ptr != calldata_end_ptr {
        let val = *vm.get_integer(calldata_start_ptr)?;
        values.push(val);
        calldata_start_ptr.offset += 1;
    }
    Ok(values)
}

pub(crate) fn get_cell_val(
    vm: &VirtualMachine,
    cell: CellRef,
) -> Result<Felt252, VirtualMachineError> {
    Ok(*vm.get_integer(cell_ref_to_relocatable(cell, vm))?.as_ref())
}

fn get_double_deref_val(
    vm: &VirtualMachine,
    cell: CellRef,
    offset: &Felt252,
) -> Result<Felt252, VirtualMachineError> {
    Ok(*vm.get_integer(get_ptr(vm, cell, offset)?)?)
}

/// Fetches the value of `res_operand` from the vm.
pub fn get_val(
    vm: &VirtualMachine,
    res_operand: &ResOperand,
) -> Result<Felt252, VirtualMachineError> {
    match res_operand {
        ResOperand::Deref(cell) => get_cell_val(vm, *cell),
        ResOperand::DoubleDeref(cell, offset) => get_double_deref_val(vm, *cell, &(*offset).into()),
        ResOperand::Immediate(x) => Ok(Felt252::from(x.value.clone())),
        ResOperand::BinOp(op) => {
            let a = get_cell_val(vm, op.a)?;
            let b = match &op.b {
                DerefOrImmediate::Deref(cell) => get_cell_val(vm, *cell)?,
                DerefOrImmediate::Immediate(x) => Felt252::from(x.value.clone()),
            };
            match op.op {
                Operation::Add => Ok(a + b),
                Operation::Mul => Ok(a * b),
            }
        }
    }
}

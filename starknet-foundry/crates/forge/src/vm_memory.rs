use anyhow::{anyhow, Result};
use cairo_felt::Felt252;
use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
use cairo_vm::vm::vm_core::VirtualMachine;
use num_traits::ToPrimitive;

pub(crate) fn write_cheatcode_panic(
    vm: &mut VirtualMachine,
    result_segment_ptr: &mut Relocatable,
    panic_data: Vec<Felt252>,
) {
    insert_at_pointer(vm, result_segment_ptr, 1).expect("Failed to insert err code");
    insert_at_pointer(vm, result_segment_ptr, panic_data.len())
        .expect("Failed to insert panic_data len");
    for datum in panic_data {
        insert_at_pointer(vm, result_segment_ptr, datum).expect("Failed to insert error in memory");
    }
}

pub(crate) fn insert_at_pointer<T: Into<MaybeRelocatable>>(
    vm: &mut VirtualMachine,
    ptr: &mut Relocatable,
    value: T,
) -> Result<()> {
    vm.insert_value(*ptr, value)?;
    *ptr += 1;
    Ok(())
}

pub(crate) fn usize_from_pointer(vm: &mut VirtualMachine, ptr: &mut Relocatable) -> Result<usize> {
    let gas_counter = vm
        .get_integer(*ptr)?
        .to_usize()
        .ok_or_else(|| anyhow!("Failed to convert to usize"))?;
    *ptr += 1;
    Ok(gas_counter)
}

pub(crate) fn relocatable_from_pointer(
    vm: &mut VirtualMachine,
    ptr: &mut Relocatable,
) -> Result<Relocatable> {
    let start = vm.get_relocatable(*ptr)?;
    *ptr += 1;
    Ok(start)
}

pub(crate) fn felt_from_pointer(vm: &mut VirtualMachine, ptr: &mut Relocatable) -> Result<Felt252> {
    let entry_point_selector = vm.get_integer(*ptr)?.into_owned();
    *ptr += 1;
    Ok(entry_point_selector)
}

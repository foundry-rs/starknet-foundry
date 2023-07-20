use cairo_felt::Felt252;
use cairo_lang_runner::casm_run::MemBuffer;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::vm_core::VirtualMachine;

pub(crate) fn write_cheatcode_panic(
    vm: &mut VirtualMachine,
    result_segment_ptr: &mut Relocatable,
    panic_data: Vec<Felt252>,
) {
    let mut buffer = MemBuffer::new(vm, *result_segment_ptr);

    buffer.write(1).expect("Failed to write err code");
    buffer
        .write(panic_data.len())
        .expect("Failed to write panic_data len");
    buffer
        .write_data(panic_data.iter())
        .expect("Failed to write error in memory");
}

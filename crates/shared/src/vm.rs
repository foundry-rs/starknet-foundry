use cairo_vm::vm::vm_core::VirtualMachine;
use std::iter::once;

pub trait VirtualMachineExt {
    /// Return the relocated pc values corresponding to each call instruction in the traceback.
    /// Returns the most recent call first.
    /// # Note
    /// Only call instructions from segment 0 (the main program segment) are included in the result.
    /// To get the relocated PC values, we add 1 to each entry in the returned list.
    ///
    /// This approach works specifically because all PCs are in segment 0. And relocation table for this
    /// segment is 1. [Ref](https://github.com/lambdaclass/cairo-vm/blob/82e465bc90f9f32a3b368e8336cc9d0963bbdca3/vm/src/vm/vm_memory/memory_segments.rs#L113)
    ///
    /// If the PCs were located in other segments, we would need to use a relocation table
    /// to compute their relocated values. However, the relocation table is not publicly
    /// accessible from within the VM, and accessing it would require replicating significant
    /// amounts of internal implementation code.
    fn get_reversed_pc_traceback(&self) -> Vec<usize>;
}

impl VirtualMachineExt for VirtualMachine {
    fn get_reversed_pc_traceback(&self) -> Vec<usize> {
        self.get_traceback_entries()
            .into_iter()
            // The cairo-vm implementation doesn't include the start location, so we add it manually.
            .chain(once((self.get_fp(), self.get_pc())))
            .rev()
            .map(|(_, pc)| pc)
            .filter(|pc| pc.segment_index == 0)
            .map(|pc| pc.offset + 1)
            .collect()
    }
}

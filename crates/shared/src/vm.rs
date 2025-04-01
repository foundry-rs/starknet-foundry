use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::vm_core::VirtualMachine;
use num_traits::cast::ToPrimitive;

pub trait VirtualMachineExt {
    /// Returns the values (fp, pc) corresponding to each call instruction in the traceback.
    /// Returns the most recent call last.
    /// TODO(#3170): Use `get_traceback_entries` from [`VirtualMachine`] once it is public.
    fn get_traceback_entries(&self) -> Vec<(Relocatable, Relocatable)>;

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
    fn get_reversed_pc_traceback(&self) -> Vec<usize> {
        self.get_traceback_entries()
            .into_iter()
            .rev()
            .map(|(_, pc)| pc)
            .filter(|pc| pc.segment_index == 0)
            .map(|pc| pc.offset + 1)
            .collect()
    }
}

impl VirtualMachineExt for VirtualMachine {
    /// implementation adapted from [here](https://github.com/starkware-libs/cairo/blob/795bee8a82ab495ee7f1856209fbc33cfd8c746d/crates/cairo-lang-runner/src/casm_run/mod.rs#L1355)
    ///
    /// notable changes:
    /// - in original code the code returns a vec of (pc, fp) we return a vec of (fp, pc) to comply with [`VirtualMachine`] `get_traceback_entries`
    /// - removed `println!` calls
    fn get_traceback_entries(&self) -> Vec<(Relocatable, Relocatable)> {
        let vm = self;

        let mut fp = vm.get_fp();
        let mut panic_traceback = vec![(fp, vm.get_pc())];
        // Fetch the fp and pc traceback entries
        loop {
            let ptr_at_offset =
                |offset: usize| (fp - offset).ok().and_then(|r| vm.get_relocatable(r).ok());
            // Get return pc.
            let Some(ret_pc) = ptr_at_offset(1) else {
                break;
            };
            // Get fp traceback.
            let Some(ret_fp) = ptr_at_offset(2) else {
                break;
            };
            if ret_fp == fp {
                break;
            }
            fp = ret_fp;

            let call_instruction = |offset: usize| -> Option<Relocatable> {
                let ptr = (ret_pc - offset).ok()?;
                let inst = vm.get_integer(ptr).ok()?;
                let inst_short = inst.to_u64()?;
                (inst_short & 0x7000_0000_0000_0000 == 0x1000_0000_0000_0000).then_some(ptr)
            };
            if let Some(call_pc) = call_instruction(1).or_else(|| call_instruction(2)) {
                panic_traceback.push((fp, call_pc));
            } else {
                break;
            }
        }
        panic_traceback.reverse();
        panic_traceback
    }
}

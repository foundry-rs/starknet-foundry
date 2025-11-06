use crate::cheatcode::execute_cheatcode_and_deserialize;

/// Gets the current step from Cairo VM during test execution
pub fn get_current_vm_step() -> u32 {
    execute_cheatcode_and_deserialize::<'get_current_vm_step', u32>(array![].span())
}

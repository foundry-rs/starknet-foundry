use crate::CheatnetState;
use cairo_vm::Felt252;

impl CheatnetState {
    pub fn set_block_hash(&mut self, block_number: u64, block_hash: Felt252) {
        self.block_hash.insert(block_number, block_hash);
    }
}

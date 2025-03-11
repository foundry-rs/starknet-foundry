use super::super::_cheatcode::execute_cheatcode_and_deserialize;

/// Changes the block hash for the given block number
/// - `block_number` - block number to be edited
/// - `value` - felt252 representing new block hash
pub fn cheat_block_hash(block_number: u64, value: felt252) {
    execute_cheatcode_and_deserialize::<'set_block_hash', ()>([block_number.into(), value].span());
}

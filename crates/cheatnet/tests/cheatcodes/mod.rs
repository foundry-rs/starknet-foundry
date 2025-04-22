use crate::common::felt_selector_from_name;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::calculate_variable_address;
use conversions::IntoConv;
use starknet_types_core::felt::Felt;

mod test_environment;

mod cheat_block_hash;
mod cheat_block_number;
mod cheat_block_timestamp;
mod cheat_caller_address;
mod cheat_execution_info;
mod cheat_sequencer_address;
mod declare;
mod deploy;
mod generate_random_felt;
mod get_class_hash;
mod load;
mod mock_call;
mod multiple_writes_same_storage;
mod precalculate_address;
mod replace_bytecode;
mod set_balance;
mod spy_events;
mod store;

pub fn map_entry_address(var_name: &str, key: &[Felt]) -> Felt {
    calculate_variable_address(felt_selector_from_name(var_name).into_(), Some(key))
}

pub fn variable_address(var_name: &str) -> Felt {
    calculate_variable_address(felt_selector_from_name(var_name).into_(), None)
}
